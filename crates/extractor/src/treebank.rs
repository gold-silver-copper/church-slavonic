//! The Old Church Slavonic treebanks: the UD PROIEL **train split** is a
//! table source under the institutional grant in `references/TERMS.md`
//! (both treebanks are CC BY-NC-SA upstream; the grant is what permits
//! derived cells to ship under the crate licence). The UD **dev/test
//! splits** and Syntacticus are EVALUATION sources (feature `checks` only):
//! every annotated token whose lemma and features name a cell of the schema
//! is a slot the accuracy harness scores — "corpus recall".
//!
//! The held-out property is structural, not policy: [`load_ud_proiel_train`]
//! reads only `*train*.conllu` files, and the dev/test and Syntacticus
//! loaders are compiled only under the `checks` feature, which the
//! `refresh-data` build (the only path that emits tables) never enables.
//!
//! # Sources
//!
//! - `ud-ocs-proiel-r2.18`: UD_Old_Church_Slavonic-PROIEL, CoNLL-U (the
//!   train/dev/test files). Features: `Case`, `Number`, `Gender` (a list
//!   attests every gender named), `Degree`, `Variant=Short` (absent = the
//!   long adjective/participle), `Person`, `Mood`, `Tense` with `Aspect`
//!   (the PROIEL conversion writes the aorist as `Tense=Past|Aspect=Perf`
//!   and the imperfect as `Tense=Past|Aspect=Imp`), `VerbForm`
//!   (`Fin`/`Part`; `Inf`, `PartRes` — the l-participle — and `Sup` are
//!   outside the schema), `PronType=Prs` for the personal pronoun.
//! - `syntacticus-20230428`: the PROIEL XML of every text whose `<source>`
//!   is `language="chu"` (Marianus, Zographensis, Suprasliensis, Psalterium
//!   Sinaiticum, Euchologium, the Kiev Missal, the Vitae, Chrabr). The
//!   ten-letter `morphology` string is person, number, tense, mood, voice,
//!   gender, case, degree, strength, inflection; strength `s` (strong) is
//!   the short form and `w` (weak) the long one.
//!
//! The dual is kept. A feature the schema has no cell for (a future, a
//! subjunctive, a passive or declined participle, a reflexive pronoun, an
//! ambiguous `Case=Dat,Gen`) skips the token and is counted by reason.

use crate::cells::{GENDERS, Pos, adj_cell, l_participle_cell, noun_cell, npron_cell, participle_cell, pronoun_cell, verb_cell};
use church_slavonic_core::grammar::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One attested cell of a treebank: the lemma the annotators gave the token,
/// the schema cell its features name, and the surface as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusSlot {
    pub lemma: String,
    pub pos: Pos,
    pub cell: usize,
    pub surface: String,
}

/// A loaded treebank: its slots and the accounting of what was left out.
#[derive(Debug, Default)]
pub struct Corpus {
    /// The README's "Recension" column for this treebank's rows.
    pub label: &'static str,
    /// The file stem of the misses report.
    pub file_label: &'static str,
    pub tokens: u64,
    pub slots: Vec<CorpusSlot>,
    pub skipped: BTreeMap<&'static str, u64>,
}

impl Corpus {
    fn skip(&mut self, reason: &'static str) {
        *self.skipped.entry(reason).or_default() += 1;
    }

    pub fn skipped_total(&self) -> u64 {
        self.skipped.values().sum()
    }
}

/// The two treebank directories under `--sources`.
pub const UD_PROIEL_SOURCE: &str = "ud-ocs-proiel-r2.18";
pub const SYNTACTICUS_SOURCE: &str = "syntacticus-20230428";

/// Unpack the one `.tar.gz` of a source directory into `artifacts_dir/
/// treebanks/<name>` (once; the unpacked tree is reused) and return it, or
/// `None` when the source is not downloaded.
fn unpacked(
    sources_dir: &Path,
    artifacts_dir: &Path,
    name: &str,
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let source = sources_dir.join(name);
    if !source.is_dir() {
        return Ok(None);
    }
    let Some(tarball) = fs::read_dir(&source)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.to_string_lossy().ends_with(".tar.gz"))
    else {
        return Ok(None);
    };
    let into = artifacts_dir.join("treebanks").join(name);
    if !into.is_dir() {
        fs::create_dir_all(&into)?;
        let status = Command::new("tar")
            .arg("xzf")
            .arg(&tarball)
            .arg("-C")
            .arg(&into)
            .status()?;
        if !status.success() {
            return Err(format!("tar failed on {}", tarball.display()).into());
        }
    }
    Ok(Some(into))
}

fn files_with_extension(
    dir: &Path,
    extension: &str,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            files_with_extension(&path, extension, out)?;
        } else if path.extension().is_some_and(|e| e == extension) {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

/// A treebank surface, lowercased and with the editors' brackets removed:
/// the transcriptions carry supplied or damaged letters as `дрѣ[вѣ]`,
/// `въ]ньмемъ`, `христ(ос)ъ`.
fn clean_surface(form: &str) -> String {
    form.to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '[' | ']' | '(' | ')'))
        .collect()
}

// ---------------------------------------------------------------------------
// UD CoNLL-U
// ---------------------------------------------------------------------------

/// The evaluation corpus: the **dev and test** splits only — the train
/// split feeds tables and must never be scored as held-out.
#[cfg(feature = "checks")]
pub fn load_ud_proiel(
    sources_dir: &Path,
    artifacts_dir: &Path,
) -> Result<Option<Corpus>, Box<dyn Error>> {
    load_ud_proiel_split(
        sources_dir,
        artifacts_dir,
        false,
        "OCS (UD PROIEL r2.18 dev+test, corpus recall)",
    )
}

/// The table-source corpus: the **train** split only (see the module docs
/// and `references/TERMS.md`).
pub fn load_ud_proiel_train(
    sources_dir: &Path,
    artifacts_dir: &Path,
) -> Result<Option<Corpus>, Box<dyn Error>> {
    load_ud_proiel_split(sources_dir, artifacts_dir, true, "OCS (UD PROIEL train)")
}

fn load_ud_proiel_split(
    sources_dir: &Path,
    artifacts_dir: &Path,
    train: bool,
    label: &'static str,
) -> Result<Option<Corpus>, Box<dyn Error>> {
    let Some(root) = unpacked(sources_dir, artifacts_dir, UD_PROIEL_SOURCE)? else {
        return Ok(None);
    };
    let mut corpus = Corpus {
        label,
        file_label: "ud_proiel",
        ..Corpus::default()
    };
    let mut files = Vec::new();
    files_with_extension(&root, "conllu", &mut files)?;
    files.retain(|f| {
        f.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.ends_with("train") == train)
    });
    for file in files {
        for line in fs::read_to_string(&file)?.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 10 || fields[0].contains('-') || fields[0].contains('.') {
                continue;
            }
            corpus.tokens += 1;
            let feats: BTreeMap<&str, &str> = fields[5]
                .split('|')
                .filter_map(|f| f.split_once('='))
                .collect();
            ud_token(&mut corpus, fields[1], fields[2], fields[3], &feats);
        }
    }
    Ok(Some(corpus))
}

fn ud_token(
    corpus: &mut Corpus,
    form: &str,
    lemma: &str,
    upos: &str,
    feats: &BTreeMap<&str, &str>,
) {
    if !matches!(upos, "NOUN" | "PROPN" | "ADJ" | "VERB" | "AUX" | "PRON") {
        return corpus.skip("part of speech outside the four tables");
    }
    let number = match feats.get("Number") {
        Some(&"Sing") => Number::Singular,
        Some(&"Dual") => Number::Dual,
        Some(&"Plur") => Number::Plural,
        _ => return corpus.skip("no number"),
    };
    let case = |corpus: &mut Corpus| match feats.get("Case") {
        Some(&"Nom") => Some(Case::Nominative),
        Some(&"Gen") => Some(Case::Genitive),
        Some(&"Dat") => Some(Case::Dative),
        Some(&"Acc") => Some(Case::Accusative),
        Some(&"Ins") => Some(Case::Instrumental),
        Some(&"Loc") => Some(Case::Locative),
        Some(&"Voc") => Some(Case::Vocative),
        Some(_) => {
            corpus.skip("ambiguous case");
            None
        }
        None => {
            corpus.skip("no case");
            None
        }
    };
    let genders: Vec<Gender> = feats
        .get("Gender")
        .map(|g| {
            g.split(',')
                .filter_map(|g| match g {
                    "Masc" => Some(Gender::Masculine),
                    "Fem" => Some(Gender::Feminine),
                    "Neut" => Some(Gender::Neuter),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let lemma = lemma.to_lowercase();
    let surface = clean_surface(form);
    match upos {
        "NOUN" | "PROPN" => {
            let Some(case) = case(corpus) else { return };
            corpus.slots.push(CorpusSlot {
                lemma,
                pos: Pos::Noun,
                cell: noun_cell(&case, &number),
                surface,
            });
        }
        "ADJ" => {
            let Some(case) = case(corpus) else { return };
            let degree = match feats.get("Degree") {
                Some(&"Cmp") => Degree::Comparative,
                Some(&"Sup") => return corpus.skip("adjective: superlative"),
                _ => Degree::Positive,
            };
            let short = feats.get("Variant") == Some(&"Short");
            let Some(lemma) = adjective_lemma(&lemma, short) else {
                return corpus.skip("adjective: no long lemma from the short one");
            };
            let genders = if genders.is_empty() {
                GENDERS.to_vec()
            } else {
                genders
            };
            for gender in genders {
                if let Some(cell) = adj_cell(&case, &number, &gender, &degree) {
                    corpus.slots.push(CorpusSlot {
                        lemma: lemma.clone(),
                        pos: Pos::Adj,
                        cell,
                        surface: surface.clone(),
                    });
                }
            }
        }
        "VERB" | "AUX" => {
            let cell = match feats.get("VerbForm") {
                Some(&"Fin") => {
                    let person = match feats.get("Person") {
                        Some(&"1") => Person::First,
                        Some(&"2") => Person::Second,
                        Some(&"3") => Person::Third,
                        _ => return corpus.skip("verb: no person"),
                    };
                    let (tense, verb_form) =
                        match (feats.get("Mood"), feats.get("Tense"), feats.get("Aspect")) {
                            (Some(&"Imp"), _, _) => (Tense::Present, Form::Imperative),
                            (Some(&"Ind"), Some(&"Pres"), _) => (Tense::Present, Form::Finite),
                            (Some(&"Ind"), Some(&"Past"), Some(&"Imp")) => {
                                (Tense::Imperfect, Form::Finite)
                            }
                            (Some(&"Ind"), Some(&"Past"), _) => (Tense::Aorist, Form::Finite),
                            (Some(&"Ind"), Some(&"Fut"), _) => return corpus.skip("verb: future"),
                            (Some(&"Sub"), _, _) => return corpus.skip("verb: subjunctive"),
                            _ => return corpus.skip("verb: finite form without a tense"),
                        };
                    verb_cell(&person, &number, &tense, &verb_form)
                }
                Some(&"Part") => {
                    let voice = if feats.get("Voice") == Some(&"Pass") {
                        Voice::Passive
                    } else {
                        Voice::Active
                    };
                    let series = if feats.get("Variant") == Some(&"Short") {
                        Series::Short
                    } else {
                        Series::Long
                    };
                    let tense = match feats.get("Tense") {
                        Some(&"Pres") => Tense::Present,
                        Some(&"Past") => Tense::Aorist,
                        _ => return corpus.skip("verb: participle without a tense"),
                    };
                    let Some(case) = case(corpus) else { return };
                    let genders = if genders.is_empty() {
                        GENDERS.to_vec()
                    } else {
                        genders
                    };
                    for gender in genders {
                        corpus.slots.push(CorpusSlot {
                            lemma: lemma.clone(),
                            pos: Pos::Verb,
                            cell: participle_cell(&voice, &series, &tense, &gender, &number, &case),
                            surface: surface.clone(),
                        });
                    }
                    return;
                }
                Some(&"Inf") => return corpus.skip("verb: infinitive (the lemma itself)"),
                Some(&"PartRes") => {
                    // The l-participle: nominative-only gender/number cells
                    // (an absent case tag reads as the nominative).
                    if matches!(feats.get("Case"), Some(c) if *c != "Nom") {
                        return corpus.skip("verb: l-participle in an oblique case");
                    }
                    let genders = if genders.is_empty() {
                        GENDERS.to_vec()
                    } else {
                        genders
                    };
                    for gender in genders {
                        corpus.slots.push(CorpusSlot {
                            lemma: lemma.clone(),
                            pos: Pos::Verb,
                            cell: l_participle_cell(&gender, &number),
                            surface: surface.clone(),
                        });
                    }
                    return;
                }
                Some(&"Sup") => return corpus.skip("verb: supine"),
                _ => return corpus.skip("verb: no verb form"),
            };
            let Some(cell) = cell else {
                return corpus.skip("verb: cell outside the schema");
            };
            corpus.slots.push(CorpusSlot {
                lemma,
                pos: Pos::Verb,
                cell,
                surface,
            });
        }
        "PRON" => {
            if feats.get("PronType") != Some(&"Prs") {
                // The non-personal pronouns: lemma-keyed gender/number/case
                // cells. An unspecified gender reads as the masculine (the
                // interrogatives answer every gender the same row).
                if feats.get("PronType") == Some(&"Rcp") {
                    return corpus.skip("pronoun: reciprocal");
                }
                let Some(case) = case(corpus) else { return };
                let genders = if genders.is_empty() {
                    vec![Gender::Masculine]
                } else {
                    genders
                };
                for gender in genders {
                    corpus.slots.push(CorpusSlot {
                        lemma: lemma.clone(),
                        pos: Pos::NPron,
                        cell: npron_cell(&gender, &number, &case),
                        surface: surface.clone(),
                    });
                }
                return;
            }
            if feats.get("Reflex") == Some(&"Yes") {
                // The reflexive's own cells of the shared row (v1.2 part 3);
                // the possessive свои carries the same feature and is the
                // non-personal pronoun's.
                if feats.get("Poss") == Some(&"Yes") {
                    return corpus.skip("pronoun: reflexive possessive");
                }
                let Some(case) = case(corpus) else { return };
                if matches!(case, Case::Vocative | Case::Nominative) {
                    return corpus.skip("pronoun: reflexive nominative");
                }
                corpus.slots.push(CorpusSlot {
                    lemma: crate::cells::PRONOUN_KEY.to_string(),
                    pos: Pos::Pronoun,
                    cell: crate::cells::reflexive_cell(&case),
                    surface,
                });
                return;
            }
            let person = match feats.get("Person") {
                Some(&"1") => Person::First,
                Some(&"2") => Person::Second,
                Some(&"3") => Person::Third,
                _ => return corpus.skip("pronoun: no person"),
            };
            let Some(case) = case(corpus) else { return };
            if case == Case::Vocative {
                return corpus.skip("pronoun: vocative");
            }
            let genders = if genders.is_empty() || person != Person::Third {
                GENDERS.to_vec()
            } else {
                genders
            };
            for gender in genders {
                corpus.slots.push(CorpusSlot {
                    lemma: crate::cells::PRONOUN_KEY.to_string(),
                    pos: Pos::Pronoun,
                    cell: pronoun_cell(&person, &number, &gender, &case),
                    surface: surface.clone(),
                });
            }
        }
        _ => corpus.skip("part of speech outside the four tables"),
    }
}

/// The table lemma of an adjective form: the treebanks cite the short
/// nominative (`новъ`); a long form is keyed by its own nominative (`новꙑи`,
/// `синии`), as in the Kaikki tables.
fn adjective_lemma(lemma: &str, short: bool) -> Option<String> {
    if short {
        return Some(lemma.to_string());
    }
    if let Some(stem) = lemma.strip_suffix('ъ') {
        Some(format!("{stem}ꙑи"))
    } else if let Some(stem) = lemma.strip_suffix('ь') {
        Some(format!("{stem}ии"))
    } else if lemma.ends_with("ꙑи") || lemma.ends_with("ии") {
        Some(lemma.to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// The train split as a table source
// ---------------------------------------------------------------------------

/// One aggregated attestation of the train split: a form observed `count`
/// times in a lemma's cell, already normalised to canonical spelling.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainRecord {
    pub pos: String,
    pub lemma: String,
    pub cell: usize,
    pub form: String,
    pub count: u64,
}

/// Is the surface written under a titlo or a supralinear letter — an
/// abbreviation whose omitted letters no mark-stripping restores? (The
/// palatalization hook and the breathings, U+0484..U+0486, mark ordinary
/// fully-written words and do not count.)
pub fn is_abbreviated(surface: &str) -> bool {
    surface.chars().any(|c| {
        matches!(c, '\u{0346}' | '\u{0483}' | '\u{0487}'..='\u{0489}' | '\u{2DE0}'..='\u{2DFF}' | '\u{A66F}')
    })
}

/// Filter the UD PROIEL train split into `out`: normalised, counted
/// attestations, one JSON line per (pos, lemma, cell, form). A token under a
/// titlo (an abbreviation, not a form), a surface or lemma that is not one
/// canonical Cyrillic word after mark-stripping and realisation, or a junk
/// lemma is skipped and counted by reason. The frequency GATE is not applied
/// here — it is extraction policy (see `extract::gather_ud_proiel`).
pub fn filter_train(
    sources_dir: &Path,
    artifacts_dir: &Path,
    out: &Path,
) -> Result<(), Box<dyn Error>> {
    use church_slavonic_core::orthography::{realise, strip_marks};
    let Some(corpus) = load_ud_proiel_train(sources_dir, artifacts_dir)? else {
        return Err(format!(
            "no UD PROIEL source under {} — download it or pass `--sources DIR`.",
            sources_dir.display()
        )
        .into());
    };
    let ocs = Recension::OldChurchSlavonic;
    let mut counts: BTreeMap<(Pos, String, usize, String), u64> = BTreeMap::new();
    let mut skipped: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut kept = 0u64;
    for slot in &corpus.slots {
        if is_abbreviated(&slot.surface) {
            *skipped.entry("token under a titlo").or_default() += 1;
            continue;
        }
        // The payerok stands for a jer the scribe superscripted (`имꙿ`).
        let form = realise(&strip_marks(&slot.surface).replace('ꙿ', "ъ"), &ocs);
        if !crate::extract::word_is_proper(&form) {
            *skipped
                .entry("surface is not one canonical word")
                .or_default() += 1;
            continue;
        }
        let lemma = if slot.pos == Pos::Pronoun {
            slot.lemma.clone()
        } else {
            let lemma = realise(&strip_marks(&slot.lemma), &ocs);
            if !crate::extract::word_is_proper(&lemma) {
                *skipped
                    .entry("lemma is not one canonical word")
                    .or_default() += 1;
                continue;
            }
            lemma
        };
        kept += 1;
        *counts
            .entry((slot.pos, lemma, slot.cell, form))
            .or_default() += 1;
    }
    let mut writer = String::new();
    for ((pos, lemma, cell, form), count) in &counts {
        writer.push_str(&serde_json::to_string(&TrainRecord {
            pos: pos.label().to_string(),
            lemma: lemma.clone(),
            cell: *cell,
            form: form.clone(),
            count: *count,
        })?);
        writer.push('\n');
    }
    fs::write(out, writer)?;
    println!(
        "Filtered UD PROIEL train: {} tokens, {} slot attestations kept ({} distinct), {} skipped:{} into {}",
        corpus.tokens,
        kept,
        counts.len(),
        skipped.values().sum::<u64>() + corpus.skipped_total(),
        skipped
            .iter()
            .map(|(reason, n)| format!(" {reason}={n};"))
            .collect::<String>(),
        out.display()
    );
    Ok(())
}

/// Read a filtered train intermediate back.
pub fn read_train(path: &Path) -> Result<Vec<TrainRecord>, Box<dyn Error>> {
    let mut out = Vec::new();
    for line in fs::read_to_string(path)?.lines() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// PROIEL XML (Syntacticus)
// ---------------------------------------------------------------------------

#[cfg(feature = "checks")]
pub fn load_syntacticus(
    sources_dir: &Path,
    artifacts_dir: &Path,
) -> Result<Option<Corpus>, Box<dyn Error>> {
    let Some(root) = unpacked(sources_dir, artifacts_dir, SYNTACTICUS_SOURCE)? else {
        return Ok(None);
    };
    let mut corpus = Corpus {
        label: "OCS (Syntacticus 2023-04-28, corpus recall)",
        file_label: "syntacticus",
        ..Corpus::default()
    };
    let mut files = Vec::new();
    files_with_extension(&root, "xml", &mut files)?;
    for file in files {
        let xml = fs::read_to_string(&file)?;
        let is_ocs = xml
            .find("<source ")
            .map(|at| &xml[at..])
            .and_then(|s| s.find('>').map(|end| &s[..end]))
            .is_some_and(|tag| tag.contains("language=\"chu\""));
        if !is_ocs {
            continue;
        }
        let mut rest = xml.as_str();
        while let Some(at) = rest.find("<token ") {
            let tag = &rest[at..];
            let Some(end) = tag.find('>') else { break };
            let tag = &tag[..end];
            rest = &rest[at + end + 1..];
            let attribute = |name: &str| {
                let key = format!(" {name}=\"");
                let start = tag.find(&key)? + key.len();
                let value = &tag[start..];
                Some(unescape(&value[..value.find('"')?]))
            };
            let (Some(form), Some(lemma), Some(pos), Some(morphology)) = (
                attribute("form"),
                attribute("lemma"),
                attribute("part-of-speech"),
                attribute("morphology"),
            ) else {
                continue;
            };
            corpus.tokens += 1;
            proiel_token(&mut corpus, &form, &lemma, &pos, &morphology);
        }
    }
    Ok(Some(corpus))
}

#[cfg_attr(not(feature = "checks"), allow(dead_code))]
fn unescape(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg_attr(not(feature = "checks"), allow(dead_code))]
fn proiel_token(corpus: &mut Corpus, form: &str, lemma: &str, pos: &str, morphology: &str) {
    if !matches!(
        pos,
        "Nb" | "Ne" | "A-" | "V-" | "Pp" | "Pk" | "Pd" | "Pi" | "Pr" | "Px" | "Ps" | "Pt" | "Py"
            | "Pc"
    ) {
        return corpus.skip("part of speech outside the four tables");
    }
    let m: Vec<char> = morphology.chars().collect();
    if m.len() < 9 {
        return corpus.skip("morphology too short");
    }
    let (person, number, tense, mood, voice, gender, case_letter, degree, strength) =
        (m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8]);
    let number = match number {
        's' => Number::Singular,
        'd' => Number::Dual,
        'p' => Number::Plural,
        _ => return corpus.skip("no number"),
    };
    let case = |corpus: &mut Corpus| match case_letter {
        'n' => Some(Case::Nominative),
        'g' => Some(Case::Genitive),
        'd' => Some(Case::Dative),
        'a' => Some(Case::Accusative),
        'i' => Some(Case::Instrumental),
        'l' => Some(Case::Locative),
        'v' => Some(Case::Vocative),
        '-' | 'z' => {
            corpus.skip("no case");
            None
        }
        _ => {
            corpus.skip("ambiguous case");
            None
        }
    };
    let genders: Vec<Gender> = match gender {
        'm' => vec![Gender::Masculine],
        'f' => vec![Gender::Feminine],
        'n' => vec![Gender::Neuter],
        'p' => vec![Gender::Masculine, Gender::Feminine],
        'o' => vec![Gender::Masculine, Gender::Neuter],
        'r' => vec![Gender::Feminine, Gender::Neuter],
        _ => GENDERS.to_vec(),
    };
    let lemma = lemma.to_lowercase();
    let surface = clean_surface(form);
    match pos {
        "Nb" | "Ne" => {
            let Some(case) = case(corpus) else { return };
            corpus.slots.push(CorpusSlot {
                lemma,
                pos: Pos::Noun,
                cell: noun_cell(&case, &number),
                surface,
            });
        }
        "A-" => {
            let Some(case) = case(corpus) else { return };
            let degree = match degree {
                'c' => Degree::Comparative,
                's' => return corpus.skip("adjective: superlative"),
                _ => Degree::Positive,
            };
            let short = match strength {
                's' => true,
                'w' => false,
                _ => return corpus.skip("adjective: strength unspecified"),
            };
            let Some(lemma) = adjective_lemma(&lemma, short) else {
                return corpus.skip("adjective: no long lemma from the short one");
            };
            for g in genders {
                if let Some(cell) = adj_cell(&case, &number, &g, &degree) {
                    corpus.slots.push(CorpusSlot {
                        lemma: lemma.clone(),
                        pos: Pos::Adj,
                        cell,
                        surface: surface.clone(),
                    });
                }
            }
        }
        "V-" => {
            let cell = match mood {
                'i' | 'm' => {
                    let person = match person {
                        '1' => Person::First,
                        '2' => Person::Second,
                        '3' => Person::Third,
                        _ => return corpus.skip("verb: no person"),
                    };
                    let (tense, verb_form) = match (mood, tense) {
                        ('m', _) => (Tense::Present, Form::Imperative),
                        ('i', 'p') => (Tense::Present, Form::Finite),
                        ('i', 'a') => (Tense::Aorist, Form::Finite),
                        ('i', 'i') => (Tense::Imperfect, Form::Finite),
                        ('i', 'f') => return corpus.skip("verb: future"),
                        _ => return corpus.skip("verb: tense outside the schema"),
                    };
                    verb_cell(&person, &number, &tense, &verb_form)
                }
                'p' => {
                    let part_voice = if voice == 'p' {
                        Voice::Passive
                    } else {
                        Voice::Active
                    };
                    let series = match strength {
                        's' => Series::Short,
                        'w' => Series::Long,
                        _ => return corpus.skip("verb: participle strength unspecified"),
                    };
                    let part_tense = match tense {
                        'p' => Tense::Present,
                        'u' | 'a' => Tense::Aorist,
                        _ => return corpus.skip("verb: participle without a tense"),
                    };
                    let Some(case) = case(corpus) else { return };
                    for gender in &genders {
                        corpus.slots.push(CorpusSlot {
                            lemma: lemma.clone(),
                            pos: Pos::Verb,
                            cell: participle_cell(
                                &part_voice,
                                &series,
                                &part_tense,
                                gender,
                                &number,
                                &case,
                            ),
                            surface: surface.clone(),
                        });
                    }
                    return;
                }
                'n' => return corpus.skip("verb: infinitive (the lemma itself)"),
                's' => return corpus.skip("verb: subjunctive"),
                _ => return corpus.skip("verb: mood outside the schema"),
            };
            let Some(cell) = cell else {
                return corpus.skip("verb: cell outside the schema");
            };
            corpus.slots.push(CorpusSlot {
                lemma,
                pos: Pos::Verb,
                cell,
                surface,
            });
        }
        "Pp" => {
            let person = match person {
                '1' => Person::First,
                '2' => Person::Second,
                '3' => Person::Third,
                _ => return corpus.skip("pronoun: no person"),
            };
            let Some(case) = case(corpus) else { return };
            if case == Case::Vocative {
                return corpus.skip("pronoun: vocative");
            }
            let genders = if person == Person::Third {
                genders
            } else {
                GENDERS.to_vec()
            };
            for g in genders {
                corpus.slots.push(CorpusSlot {
                    lemma: crate::cells::PRONOUN_KEY.to_string(),
                    pos: Pos::Pronoun,
                    cell: pronoun_cell(&person, &number, &g, &case),
                    surface: surface.clone(),
                });
            }
        }
        "Pd" | "Pi" | "Pr" | "Px" | "Ps" | "Pt" | "Py" => {
            let Some(case) = case(corpus) else { return };
            let genders = if gender == '-' {
                vec![Gender::Masculine]
            } else {
                genders
            };
            for g in genders {
                corpus.slots.push(CorpusSlot {
                    lemma: lemma.clone(),
                    pos: Pos::NPron,
                    cell: npron_cell(&g, &number, &case),
                    surface: surface.clone(),
                });
            }
        }
        "Pc" => corpus.skip("pronoun: reciprocal"),
        "Pk" => {
            let Some(case) = case(corpus) else { return };
            if matches!(case, Case::Vocative | Case::Nominative) {
                return corpus.skip("pronoun: reflexive nominative");
            }
            corpus.slots.push(CorpusSlot {
                lemma: crate::cells::PRONOUN_KEY.to_string(),
                pos: Pos::Pronoun,
                cell: crate::cells::reflexive_cell(&case),
                surface: surface.clone(),
            });
        }
        _ => corpus.skip("part of speech outside the four tables"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ud_features_map_onto_the_schema_and_the_rest_is_counted() {
        let mut corpus = Corpus::default();
        fn feats(s: &str) -> BTreeMap<&str, &str> {
            s.split('|').filter_map(|f| f.split_once('=')).collect()
        }
        let noun = feats("Case=Gen|Gender=Fem|Number=Sing");
        ud_token(&mut corpus, "мѫченіцѧ", "мѫчѣница", "NOUN", &noun);
        let adj = feats("Case=Gen|Degree=Pos|Gender=Fem|Number=Sing");
        ud_token(&mut corpus, "блаженъиѧ", "блаженъ", "ADJ", &adj);
        let imp = feats("Mood=Imp|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin|Voice=Act");
        ud_token(&mut corpus, "Подазь", "подати", "VERB", &imp);
        let aor =
            feats("Aspect=Perf|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin|Voice=Act");
        ud_token(&mut corpus, "рече", "решти", "VERB", &aor);
        let ipf =
            feats("Aspect=Imp|Mood=Ind|Number=Plur|Person=3|Tense=Past|VerbForm=Fin|Voice=Act");
        ud_token(&mut corpus, "глаголаахѫ", "глаголати", "VERB", &ipf);
        let pron = feats("Case=Dat|Gender=Fem,Masc|Number=Plur|Person=1|PronType=Prs");
        ud_token(&mut corpus, "намъ", "мꙑ", "PRON", &pron);
        let res = feats("Gender=Masc|Number=Sing|Tense=Past|VerbForm=PartRes|Voice=Act");
        ud_token(&mut corpus, "далъ", "дати", "VERB", &res);
        let ambiguous = feats("Case=Dat,Gen|Gender=Masc|Number=Sing");
        ud_token(&mut corpus, "x", "x", "NOUN", &ambiguous);
        let cells: Vec<(Pos, usize, &str)> = corpus
            .slots
            .iter()
            .map(|s| (s.pos, s.cell, s.lemma.as_str()))
            .collect();
        assert_eq!(cells[0], (Pos::Noun, 1, "мѫчѣница"));
        assert_eq!(
            cells[1],
            (
                Pos::Adj,
                adj_cell(
                    &Case::Genitive,
                    &Number::Singular,
                    &Gender::Feminine,
                    &Degree::Positive
                )
                .expect("cell"),
                "блаженꙑи"
            )
        );
        assert_eq!(cells[2], (Pos::Verb, 28, "подати"));
        assert_eq!(cells[3], (Pos::Verb, 20, "решти"));
        assert_eq!(cells[4], (Pos::Verb, 17, "глаголати"));
        assert_eq!(
            cells[5].1,
            pronoun_cell(
                &Person::First,
                &Number::Plural,
                &Gender::Masculine,
                &Case::Dative
            )
        );
        assert_eq!(
            cells[8],
            (
                Pos::Verb,
                l_participle_cell(&Gender::Masculine, &Number::Singular),
                "дати"
            )
        );
        assert_eq!(cells.len(), 9);
        assert_eq!(corpus.skipped.get("ambiguous case"), Some(&1));
    }

    #[test]
    fn proiel_morphology_maps_onto_the_schema() {
        let mut corpus = Corpus::default();
        proiel_token(&mut corpus, "остави", "оставити", "V-", "2spma----i");
        proiel_token(&mut corpus, "шедъ", "ити", "V-", "-supamn-si");
        proiel_token(&mut corpus, "даръ", "даръ", "Nb", "-s---ma--i");
        proiel_token(&mut corpus, "новꙑи", "новъ", "A-", "-s---mnpwi");
        proiel_token(&mut corpus, "себе", "себе", "Pk", "3s---qa--i");
        proiel_token(&mut corpus, "ѥмоу", "и", "Pp", "3s---md--i");
        let cells: Vec<(Pos, usize, &str)> = corpus
            .slots
            .iter()
            .map(|s| (s.pos, s.cell, s.lemma.as_str()))
            .collect();
        assert_eq!(cells[0], (Pos::Verb, 28, "оставити"));
        // The declined-participle cell: short active past, m nom sg (38 + 63).
        assert_eq!(cells[1], (Pos::Verb, 101, "ити"));
        assert_eq!(cells[2], (Pos::Noun, 3, "даръ"));
        assert_eq!(cells[3].2, "новꙑи");
        assert_eq!(cells[4], (Pos::Pronoun, crate::cells::reflexive_cell(&Case::Accusative), "personal"));
        assert_eq!(
            cells[5],
            (
                Pos::Pronoun,
                pronoun_cell(
                    &Person::Third,
                    &Number::Singular,
                    &Gender::Masculine,
                    &Case::Dative
                ),
                "personal"
            )
        );
        // the reflexive's own cell of the shared row (accusative 93)
        assert!(cells.iter().any(|c| *c == (Pos::Pronoun, crate::cells::reflexive_cell(&Case::Accusative), "personal")));
        assert_eq!(corpus.skipped.get("pronoun: reflexive"), None);
    }
}
