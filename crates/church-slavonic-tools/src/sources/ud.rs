//! Legacy UD/PROIEL adapters into the current inflection cell schema.
//! Original annotations and unsupported distinctions are not yet fully retained.
//! Filename-based train/dev/test selection does not establish independence:
//! consumers include import, training and regression diagnostics, and overlapping
//! source ancestry requires a separate audit. Local archive identity establishes
//! supplied bytes only, not linguistic authority, authenticity or permission.
//!
//! Archives pass through the bounded, content-verified staged cache in archive.rs.
//! Mapping and skip behavior below remain legacy behavior pending migration.

use church_slavonic::cell::{AdjCell, Cell, FiniteTense, NounCell, PartTense, Pos, PronCell, VerbCell};
use church_slavonic::grammar::*;
use church_slavonic::orthography::{realise, strip_marks};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// One attested cell of a treebank: the lemma the annotators gave the
/// token, the typed cell its features name, and the surface as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusSlot {
    pub lemma: String,
    pub pos: Pos,
    pub cell: Cell,
    pub surface: String,
}

/// One token of a sentence in reading order: its surface and the slot
/// indices its gold reading occupies (several for an adjective whose
/// gender the annotation leaves open; none for a token outside the
/// lexicon's parts of speech).
#[derive(Debug, Clone)]
pub struct SequenceToken {
    pub surface: String,
    /// the annotated lemma, whatever the part of speech
    pub lemma: String,
    /// a direct object (UD `obj`, PROIEL `obj`)
    pub object: bool,
    pub slots: Vec<usize>,
    /// Original record, when provided by the adapter; never an inference feature.
    pub source_record: Option<usize>,
}

/// A loaded treebank: its slots and the accounting of what was left out;
/// `sentences` the tokens in order (the tagger's training and scoring
/// material), each token pointing at its slots.
#[derive(Debug, Default)]
pub struct Corpus {
    pub label: &'static str,
    pub tokens: u64,
    pub slots: Vec<CorpusSlot>,
    pub skipped: BTreeMap<&'static str, u64>,
    pub sentences: Vec<Vec<SequenceToken>>,
    pub observations: super::observations::Observations,
    pub excluded_sources: Vec<super::observations::ExcludedDocument>,
}

impl Corpus {
    /// Export original records and their legacy mapping as separate JSONL rows.
    pub fn write_observations(&self, mut out: impl std::io::Write) -> std::io::Result<()> {
        if self.observations.documents().is_empty() && self.excluded_sources.is_empty() { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"original corpus records unavailable")); }
        serde_json::to_writer(&mut out, &serde_json::json!({"schema":1,"type":"mapping_policy","definition":"legacy-corpus-cell-mapping-v2","corpus":self.label,"tokens":self.tokens,"skipped":self.skipped,"limitations":"Original lines are preserved; mapped labels retain legacy category folds, lowercased lemmas and cleaned surfaces. Neither annotations nor mappings are independently adjudicated gold."}))?;
        writeln!(out)?;
        for document in &self.excluded_sources {
            serde_json::to_writer(&mut out, &serde_json::json!({"schema":1,"type":"excluded_source","document":document}))?;
            writeln!(out)?;
        }
        self.observations.write_jsonl(&mut out)?;
        for (id, slot) in self.slots.iter().enumerate() {
            serde_json::to_writer(&mut out, &serde_json::json!({"schema":1,"type":"mapped_slot","id":id,"pos":slot.pos.tag(),"cell":slot.cell.name(),"lemma":slot.lemma,"surface":slot.surface}))?;
            writeln!(out)?;
        }
        Ok(())
    }

    pub(crate) fn skip(&mut self, reason: &'static str) {
        *self.skipped.entry(reason).or_default() += 1;
    }

    pub fn skipped_total(&self) -> u64 {
        self.skipped.values().sum()
    }
}

/// The two treebank directories under the sources directory.
pub const UD_PROIEL_SOURCE: &str = "ud-ocs-proiel-r2.18";
pub const SYNTACTICUS_SOURCE: &str = "syntacticus-20230428";

/// Resolve exactly one source archive through the verified extraction cache.
fn unpacked(sources_dir: &Path, artifacts_dir: &Path, name: &str) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let source = sources_dir.join(name);
    let entries = match fs::read_dir(&source) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut archives = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.to_string_lossy().ends_with(".tar.gz") { archives.push(path); }
    }
    if archives.is_empty() { return Err(format!("source directory {} contains no archive", source.display()).into()); }
    if archives.len() != 1 { return Err(format!("source directory {} contains multiple archives", source.display()).into()); }
    Ok(Some(super::archive::verified_extract(&archives[0], &artifacts_dir.join("treebanks-verified-v1").join(name))?))
}

fn files_with_extension(dir: &Path, extension: &str, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
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

/// A treebank surface, lowercased and with the editors' brackets removed
/// (`дрѣ[вѣ]`, `въ]ньмемъ`, `христ(ос)ъ`).
pub(crate) fn clean_surface(form: &str) -> String {
    form.to_lowercase().chars().filter(|c| !matches!(c, '[' | ']' | '(' | ')')).collect()
}

const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];

// ---------------------------------------------------------------------------
// UD CoNLL-U
// ---------------------------------------------------------------------------

/// The held-out corpus: the **dev and test** splits only.
pub fn load_ud_proiel_heldout(sources_dir: &Path, artifacts_dir: &Path) -> Result<Option<Corpus>, Box<dyn Error>> {
    load_ud_proiel_split(sources_dir, artifacts_dir, false, "OCS (UD PROIEL r2.18 dev+test)")
}

/// The import corpus: the **train** split only.
pub fn load_ud_proiel_train(sources_dir: &Path, artifacts_dir: &Path) -> Result<Option<Corpus>, Box<dyn Error>> {
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
    let mut corpus = Corpus { label, ..Corpus::default() };
    let mut files = Vec::new();
    files_with_extension(&root, "conllu", &mut files)?;
    files.retain(|f| f.file_stem().and_then(|s| s.to_str()).is_some_and(|s| s.ends_with("train") == train));
    for file in files {
        let mut text = String::new();
        use std::io::Read;
        fs::File::open(&file)?.take(64 * 1024 * 1024 + 1).read_to_string(&mut text)?;
        append_conllu(&mut corpus, file.strip_prefix(&root)?.to_string_lossy().into_owned(), text)?;
    }
    Ok(Some(corpus))
}

/// Preserve the complete source before applying legacy cell mappings. Invalid
/// rows remain explicit records; structurally valid word rows remain tokens even
/// when their feature mapping fails. Raw labels are never overwritten.
pub fn append_conllu(corpus: &mut Corpus, path: String, text: String) -> Result<(), Box<dyn Error>> {
    use super::observations::{SourceDocument, SourceRecord, RecordKind};
    let document = SourceDocument::new(path, text)?;
    let mut records = Vec::new();
    let slot_base = corpus.slots.len();
    let mut parsed = Corpus::default();
    let mut sentences = Vec::new();
    let mut sentence = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut offset = 0;
    let first_record = corpus.observations.records().len();
    for raw in document.text().split_inclusive('\n') {
        if raw.len() > 64 * 1024 { return Err("source record byte limit".into()); }
        if records.len() >= 1_000_000 { return Err("source record limit".into()); }
        let line = raw.trim_end_matches(['\r', '\n']);
        let fields: Vec<_> = line.split('\t').collect();
        let id = fields[0];
        let positive = |s: &str| !s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit()) && s.parse::<u64>().is_ok_and(|n| n > 0);
        let kind = if line.is_empty() { RecordKind::Separator }
            else if line.starts_with('#') { RecordKind::Comment }
            else if fields.len() != 10 { RecordKind::Malformed }
            else if positive(id) { RecordKind::Word }
            else if id.split_once('-').is_some_and(|(a,b)| positive(a) && positive(b) && a.parse::<u64>().ok() < b.parse::<u64>().ok()) { RecordKind::Multiword }
            else if id.split_once('.').is_some_and(|(a,b)| (a=="0" || positive(a)) && positive(b)) { RecordKind::EmptyNode }
            else { RecordKind::Malformed };
        let mut record = SourceRecord { source:0, start:offset, end:offset+raw.len(), kind, mapped_slots:Vec::new(), mapping_issue:None };
        offset += raw.len();
        if kind == RecordKind::Separator {
            if !sentence.is_empty() { sentences.push(std::mem::take(&mut sentence)); }
            ids.clear();
        } else if kind == RecordKind::Malformed {
            record.mapping_issue = Some("invalid CoNLL-U field count or ID".into());
        } else if kind == RecordKind::Word {
            parsed.tokens += 1;
            let before = parsed.slots.len();
            let old_skips = parsed.skipped.clone();
            let mut feats = BTreeMap::new();
            let valid_features = fields[5] == "_" || fields[5].split('|').all(|f| f.split_once('=').is_some_and(|(k,v)| !k.is_empty() && !v.is_empty() && feats.insert(k,v).is_none()));
            if !ids.insert(id.parse::<u64>()?) {
                parsed.skip("duplicate token ID");
            } else if !valid_features {
                parsed.skip("invalid feature field");
            } else {
                ud_token(&mut parsed, fields[1], fields[2], fields[3], &feats);
            }
            record.mapped_slots = (before..parsed.slots.len()).map(|i| slot_base+i).collect();
            record.mapping_issue = parsed.skipped.iter().find(|(k,v)| **v > old_skips.get(*k).copied().unwrap_or(0)).map(|(k,_)| k.to_string());
            if record.mapped_slots.is_empty() && record.mapping_issue.is_none() {
                parsed.skip("no mapped cells");
                record.mapping_issue = Some("no mapped cells".into());
            }
            sentence.push(SequenceToken { surface:clean_surface(fields[1]), lemma:fields[2].to_lowercase(), object:fields[7]=="obj", slots:record.mapped_slots.clone(), source_record:Some(first_record+records.len()) });
        }
        records.push(record);
    }
    if !sentence.is_empty() { sentences.push(sentence); }
    corpus.observations.append(document, records)?;
    corpus.tokens += parsed.tokens;
    corpus.slots.extend(parsed.slots);
    for (reason, count) in parsed.skipped { *corpus.skipped.entry(reason).or_default() += count; }
    corpus.sentences.extend(sentences);
    Ok(())
}

fn push(corpus: &mut Corpus, lemma: &str, pos: Pos, cell: impl Into<Cell>, surface: &str) {
    corpus.slots.push(CorpusSlot {
        lemma: lemma.to_string(),
        pos,
        cell: cell.into(),
        surface: surface.to_string(),
    });
}

fn ud_token(corpus: &mut Corpus, form: &str, lemma: &str, upos: &str, feats: &BTreeMap<&str, &str>) {
    if !matches!(upos, "NOUN" | "PROPN" | "ADJ" | "VERB" | "AUX" | "PRON") {
        return corpus.skip("part of speech outside the lexicon");
    }
    // These source categories carry no agreement number in the target cell.
    // Original tense/voice/case annotations remain on their source records.
    if matches!(upos, "VERB" | "AUX") {
        let invariant = match feats.get("VerbForm") {
            Some(&"Inf") => Some(VerbCell::Infinitive),
            Some(&"Sup") => Some(VerbCell::Supine),
            _ => None,
        };
        if let Some(cell) = invariant {
            push(corpus, &lemma.to_lowercase(), Pos::Verb, cell, &clean_surface(form));
            return;
        }
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
            push(corpus, &lemma, Pos::Noun, NounCell { case, number }, &surface);
        }
        "ADJ" => {
            let Some(case) = case(corpus) else { return };
            let degree = match feats.get("Degree") {
                Some(&"Cmp") => Degree::Comparative,
                Some(&"Sup") => Degree::Superlative,
                _ => Degree::Positive,
            };
            let series = if feats.get("Variant") == Some(&"Short") { Series::Short } else { Series::Long };
            let genders = if genders.is_empty() { GENDERS.to_vec() } else { genders };
            for gender in genders {
                push(
                    corpus,
                    &lemma,
                    Pos::Adjective,
                    AdjCell { series: Some(series), degree, gender, number, case },
                    &surface,
                );
            }
        }
        "VERB" | "AUX" => {
            let person = || match feats.get("Person") {
                Some(&"1") => Some(Person::First),
                Some(&"2") => Some(Person::Second),
                Some(&"3") => Some(Person::Third),
                _ => None,
            };
            match feats.get("VerbForm") {
                Some(&"Fin") => {
                    let Some(person) = person() else { return corpus.skip("verb: no person") };
                    let cell = match (feats.get("Mood"), feats.get("Tense"), feats.get("Aspect")) {
                        (Some(&"Imp"), _, _) => VerbCell::Imperative { person, number },
                        (Some(&"Ind"), Some(&"Pres"), _) => {
                            VerbCell::Finite { tense: FiniteTense::Present, person, number }
                        }
                        (Some(&"Ind"), Some(&"Past"), Some(&"Imp")) => {
                            VerbCell::Finite { tense: FiniteTense::Imperfect, person, number }
                        }
                        (Some(&"Ind"), Some(&"Past"), _) => {
                            VerbCell::Finite { tense: FiniteTense::Aorist, person, number }
                        }
                        (Some(&"Ind"), Some(&"Fut"), _) => {
                            VerbCell::Finite { tense: FiniteTense::Future, person, number }
                        }
                        (Some(&"Sub"), _, _) => return corpus.skip("verb: subjunctive"),
                        _ => return corpus.skip("verb: finite form without a tense"),
                    };
                    push(corpus, &lemma, Pos::Verb, cell, &surface);
                }
                Some(&"Part") => {
                    let voice = if feats.get("Voice") == Some(&"Pass") { Voice::Passive } else { Voice::Active };
                    let series = if feats.get("Variant") == Some(&"Short") { Series::Short } else { Series::Long };
                    let tense = match feats.get("Tense") {
                        Some(&"Pres") => PartTense::Present,
                        Some(&"Past") => PartTense::Past,
                        _ => return corpus.skip("verb: participle without a tense"),
                    };
                    let Some(case) = case(corpus) else { return };
                    let genders = if genders.is_empty() { GENDERS.to_vec() } else { genders };
                    for gender in genders {
                        push(
                            corpus,
                            &lemma,
                            Pos::Verb,
                            VerbCell::Participle { tense, voice, series, gender, number, case },
                            &surface,
                        );
                    }
                }
                Some(&"PartRes") => {
                    if matches!(feats.get("Case"), Some(c) if *c != "Nom") {
                        return corpus.skip("verb: l-participle in an oblique case");
                    }
                    let genders = if genders.is_empty() { GENDERS.to_vec() } else { genders };
                    for gender in genders {
                        push(corpus, &lemma, Pos::Verb, VerbCell::LPart { gender, number }, &surface);
                    }
                }
                _ => corpus.skip("verb: no verb form"),
            }
        }
        "PRON" => {
            let Some(case) = case(corpus) else { return };
            if feats.get("PronType") != Some(&"Prs") {
                if feats.get("PronType") == Some(&"Rcp") {
                    return corpus.skip("pronoun: reciprocal");
                }
                // non-personal: gender/number/case, the masculine where
                // the annotation names no gender
                let genders = if genders.is_empty() { vec![Gender::Masculine] } else { genders };
                for gender in genders {
                    let cell = PronCell { clitic: false, person: None, gender: Some(gender), number: Some(number), case };
                    push(corpus, &lemma, Pos::Pronoun, cell, &surface);
                }
                return;
            }
            if feats.get("Reflex") == Some(&"Yes") {
                if feats.get("Poss") == Some(&"Yes") {
                    return corpus.skip("pronoun: reflexive possessive");
                }
                if matches!(case, Case::Vocative | Case::Nominative) {
                    return corpus.skip("pronoun: reflexive nominative");
                }
                let cell = PronCell { clitic: false, person: None, gender: None, number: None, case };
                push(corpus, &lemma, Pos::Pronoun, cell, &surface);
                return;
            }
            let person = match feats.get("Person") {
                Some(&"1") => Person::First,
                Some(&"2") => Person::Second,
                Some(&"3") => Person::Third,
                _ => return corpus.skip("pronoun: no person"),
            };
            if case == Case::Vocative {
                return corpus.skip("pronoun: vocative");
            }
            let genders: Vec<Option<Gender>> = if person != Person::Third {
                vec![None]
            } else if genders.is_empty() {
                GENDERS.iter().copied().map(Some).collect()
            } else {
                genders.into_iter().map(Some).collect()
            };
            for gender in genders {
                let cell = PronCell { clitic: false, person: Some(person), gender, number: Some(number), case };
                push(corpus, &lemma, Pos::Pronoun, cell, &surface);
            }
        }
        _ => corpus.skip("part of speech outside the lexicon"),
    }
}

// ---------------------------------------------------------------------------
// The train split as an import source
// ---------------------------------------------------------------------------

/// One aggregated attestation of the train split: a form observed `count`
/// times in a lemma's cell, normalised to canonical OCS spelling.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainRecord {
    pub pos: String,
    pub lemma: String,
    /// The cell's name (`crate::cell` grammar).
    pub cell: String,
    pub form: String,
    pub count: u64,
}

/// Is the surface written under a titlo or a supralinear letter — an
/// abbreviation whose omitted letters no mark-stripping restores?
pub fn is_abbreviated(surface: &str) -> bool {
    surface.chars().any(|c| {
        matches!(c, '\u{0346}' | '\u{0483}' | '\u{0487}'..='\u{0489}' | '\u{2DE0}'..='\u{2DFF}' | '\u{A66F}')
    })
}

/// One canonical Cyrillic word: letters only, no `_`, no digits, no space.
pub fn word_is_proper(word: &str) -> bool {
    !word.is_empty()
        && word.chars().all(|c| {
            c.is_alphabetic()
                && matches!(c as u32, 0x0400..=0x04ff | 0x0500..=0x052f | 0x2de0..=0x2dff | 0xa640..=0xa69f | 0x1c80..=0x1c8f)
        })
}

/// Filter the UD PROIEL train split into `out`: normalised, counted
/// attestations, one JSON line per (pos, lemma, cell, form).
pub fn filter_train(sources_dir: &Path, artifacts_dir: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    let Some(corpus) = load_ud_proiel_train(sources_dir, artifacts_dir)? else {
        return Err(format!("no UD PROIEL source under {}", sources_dir.display()).into());
    };
    let ocs = Recension::OldChurchSlavonic;
    let mut counts: BTreeMap<(String, String, String, String), u64> = BTreeMap::new();
    let mut skipped: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut kept = 0u64;
    for slot in &corpus.slots {
        if is_abbreviated(&slot.surface) {
            *skipped.entry("token under a titlo").or_default() += 1;
            continue;
        }
        // The payerok stands for a jer the scribe superscripted (`имꙿ`).
        let form = realise(&strip_marks(&slot.surface).replace('ꙿ', "ъ"), &ocs);
        if !word_is_proper(&form) {
            *skipped.entry("surface is not one canonical word").or_default() += 1;
            continue;
        }
        let lemma = realise(&strip_marks(&slot.lemma), &ocs);
        if !word_is_proper(&lemma) {
            *skipped.entry("lemma is not one canonical word").or_default() += 1;
            continue;
        }
        kept += 1;
        *counts.entry((slot.pos.tag().to_string(), lemma, slot.cell.name(), form)).or_default() += 1;
    }
    let mut writer = String::new();
    for ((pos, lemma, cell, form), count) in &counts {
        writer.push_str(&serde_json::to_string(&TrainRecord {
            pos: pos.clone(),
            lemma: lemma.clone(),
            cell: cell.clone(),
            form: form.clone(),
            count: *count,
        })?);
        writer.push('\n');
    }
    fs::write(out, writer)?;
    println!(
        "Filtered UD PROIEL train: {} tokens, {} slot attestations kept ({} distinct), {} skipped into {}",
        corpus.tokens,
        kept,
        counts.len(),
        skipped.values().sum::<u64>() + corpus.skipped_total(),
        out.display()
    );
    Ok(())
}

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

pub fn load_syntacticus(sources_dir: &Path, artifacts_dir: &Path) -> Result<Option<Corpus>, Box<dyn Error>> {
    let Some(root) = unpacked(sources_dir, artifacts_dir, SYNTACTICUS_SOURCE)? else {
        return Ok(None);
    };
    let mut corpus = Corpus { label: "OCS (Syntacticus 2023-04-28)", ..Corpus::default() };
    let mut files = Vec::new();
    files_with_extension(&root, "xml", &mut files)?;
    for file in files {
        use std::io::Read;
        let mut xml = String::new();
        fs::File::open(&file)?.take(64 * 1024 * 1024 + 1).read_to_string(&mut xml)?;
        super::proiel::append_xml(&mut corpus, file.strip_prefix(&root)?.to_string_lossy().into_owned(), xml)?;
    }
    Ok(Some(corpus))
}

pub(crate) fn proiel_token(corpus: &mut Corpus, form: &str, lemma: &str, pos: &str, morphology: &str) {
    if !matches!(
        pos,
        "Nb" | "Ne" | "A-" | "V-" | "Pp" | "Pk" | "Pd" | "Pi" | "Pr" | "Px" | "Ps" | "Pt" | "Py" | "Pc"
    ) {
        return corpus.skip("part of speech outside the lexicon");
    }
    let m: Vec<char> = morphology.chars().collect();
    if m.len() < 9 {
        return corpus.skip("morphology too short");
    }
    let (person, number, tense, mood, voice, gender, case_letter, degree, strength) =
        (m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8]);
    if pos == "V-" && matches!(mood, 'n' | 'u') {
        let cell = if mood == 'n' { VerbCell::Infinitive } else { VerbCell::Supine };
        push(corpus, &lemma.to_lowercase(), Pos::Verb, cell, &clean_surface(form));
        return;
    }
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
    let person_of = |c: char| match c {
        '1' => Some(Person::First),
        '2' => Some(Person::Second),
        '3' => Some(Person::Third),
        _ => None,
    };
    let lemma = lemma.to_lowercase();
    let surface = clean_surface(form);
    match pos {
        "Nb" | "Ne" => {
            let Some(case) = case(corpus) else { return };
            push(corpus, &lemma, Pos::Noun, NounCell { case, number }, &surface);
        }
        "A-" => {
            let Some(case) = case(corpus) else { return };
            let degree = match degree {
                'c' => Degree::Comparative,
                's' => Degree::Superlative,
                _ => Degree::Positive,
            };
            let series = match strength {
                's' => Series::Short,
                'w' => Series::Long,
                _ => return corpus.skip("adjective: strength unspecified"),
            };
            for gender in genders {
                push(corpus, &lemma, Pos::Adjective, AdjCell { series: Some(series), degree, gender, number, case }, &surface);
            }
        }
        "V-" => match mood {
            'i' | 'm' => {
                let Some(person) = person_of(person) else { return corpus.skip("verb: no person") };
                let cell = match (mood, tense) {
                    ('m', _) => VerbCell::Imperative { person, number },
                    ('i', 'p') => VerbCell::Finite { tense: FiniteTense::Present, person, number },
                    ('i', 'a') => VerbCell::Finite { tense: FiniteTense::Aorist, person, number },
                    ('i', 'i') => VerbCell::Finite { tense: FiniteTense::Imperfect, person, number },
                    ('i', 'f') => VerbCell::Finite { tense: FiniteTense::Future, person, number },
                    _ => return corpus.skip("verb: tense outside the schema"),
                };
                push(corpus, &lemma, Pos::Verb, cell, &surface);
            }
            'p' => {
                let voice = if voice == 'p' { Voice::Passive } else { Voice::Active };
                let series = match strength {
                    's' => Series::Short,
                    'w' => Series::Long,
                    _ => return corpus.skip("verb: participle strength unspecified"),
                };
                let tense = match tense {
                    'p' => PartTense::Present,
                    'u' | 'a' => PartTense::Past,
                    _ => return corpus.skip("verb: participle without a tense"),
                };
                let Some(case) = case(corpus) else { return };
                for gender in genders {
                    push(corpus, &lemma, Pos::Verb, VerbCell::Participle { tense, voice, series, gender, number, case }, &surface);
                }
            }
            's' => corpus.skip("verb: subjunctive"),
            _ => corpus.skip("verb: mood outside the schema"),
        },
        "Pp" => {
            let Some(person) = person_of(person) else { return corpus.skip("pronoun: no person") };
            let Some(case) = case(corpus) else { return };
            if case == Case::Vocative {
                return corpus.skip("pronoun: vocative");
            }
            let genders: Vec<Option<Gender>> = if person == Person::Third {
                genders.into_iter().map(Some).collect()
            } else {
                vec![None]
            };
            for gender in genders {
                push(corpus, &lemma, Pos::Pronoun, PronCell { clitic: false, person: Some(person), gender, number: Some(number), case }, &surface);
            }
        }
        "Pd" | "Pi" | "Pr" | "Px" | "Ps" | "Pt" | "Py" => {
            let Some(case) = case(corpus) else { return };
            let genders = if gender == '-' { vec![Gender::Masculine] } else { genders };
            for g in genders {
                push(corpus, &lemma, Pos::Pronoun, PronCell { clitic: false, person: None, gender: Some(g), number: Some(number), case }, &surface);
            }
        }
        "Pc" => corpus.skip("pronoun: reciprocal"),
        "Pk" => {
            let Some(case) = case(corpus) else { return };
            if matches!(case, Case::Vocative | Case::Nominative) {
                return corpus.skip("pronoun: reflexive nominative");
            }
            push(corpus, &lemma, Pos::Pronoun, PronCell { clitic: false, person: None, gender: None, number: None, case }, &surface);
        }
        _ => corpus.skip("part of speech outside the lexicon"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feats(s: &str) -> BTreeMap<&str, &str> {
        s.split('|').filter_map(|f| f.split_once('=')).collect()
    }

    #[test]
    fn ud_features_map_onto_typed_cells_and_the_rest_is_counted() {
        let mut corpus = Corpus::default();
        ud_token(&mut corpus, "Рабомъ", "рабъ", "NOUN", &feats("Case=Dat|Number=Plur"));
        ud_token(&mut corpus, "бѣ", "бꙑти", "AUX", &feats("Mood=Ind|Number=Sing|Person=3|Tense=Past|Aspect=Imp|VerbForm=Fin"));
        ud_token(&mut corpus, "несꙑ", "нести", "VERB", &feats("Case=Nom|Gender=Masc|Number=Sing|Tense=Pres|VerbForm=Part|Variant=Short"));
        ud_token(&mut corpus, "мене", "азъ", "PRON", &feats("Case=Gen|Number=Sing|Person=1|PronType=Prs"));
        ud_token(&mut corpus, "себе", "себе", "PRON", &feats("Case=Gen|Number=Sing|PronType=Prs|Reflex=Yes"));
        ud_token(&mut corpus, "того", "тъ", "PRON", &feats("Case=Gen|Gender=Masc|Number=Sing|PronType=Dem"));
        ud_token(&mut corpus, "бꙑ", "бꙑти", "AUX", &feats("Mood=Sub|Number=Sing|Person=3|VerbForm=Fin"));
        ud_token(&mut corpus, "и", "и", "CCONJ", &feats("_"));
        let cells: Vec<String> = corpus.slots.iter().map(|s| s.cell.name()).collect();
        assert_eq!(cells, ["dat.pl", "impf.3.sg", "part.pres.act.short.m.sg.nom", "1.sg.gen", "gen", "m.sg.gen"]);
        assert_eq!(corpus.slots[0].surface, "рабомъ");
        assert_eq!(corpus.skipped.get("verb: subjunctive"), Some(&1));
        assert_eq!(corpus.skipped.get("part of speech outside the lexicon"), Some(&1));
    }

    #[test]
    fn proiel_morphology_maps_onto_typed_cells() {
        let mut corpus = Corpus::default();
        proiel_token(&mut corpus, "рабъ", "рабъ", "Nb", "-s---mn--i");
        proiel_token(&mut corpus, "несе", "нести", "V-", "3saia----i");
        proiel_token(&mut corpus, "добра", "добръ", "A-", "-s---fnps-");
        proiel_token(&mut corpus, "ны", "азъ", "Pp", "1p---ma--i");
        let cells: Vec<String> = corpus.slots.iter().map(|s| s.cell.name()).collect();
        assert_eq!(cells, ["nom.sg", "aor.3.sg", "short.pos.f.sg.nom", "1.pl.acc"]);
    }
}
