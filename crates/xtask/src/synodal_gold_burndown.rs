//! The gold-gap burn-down inner loop (`docs/GOLD_GAP_BURNDOWN_PROMPT.md`,
//! Slice 0): `synodal-gold propose` turns gap types into (lemma, class)
//! hypotheses clustered by the attested cells they would clear;
//! `synodal-gold admit` writes a batch of hypotheses into the curated data,
//! regenerates the registry artifact, installs it in-process (no recompile),
//! scoped-replays the batch, and keeps only the hypotheses whose class
//! reproduces every attested cell of their cluster — accents included — while
//! reverting the rest into `reports/synodal-gold-rejected-hypotheses.tsv`;
//! `synodal-gold loop` chains propose → admit → scoped replay and prints one
//! line.
//!
//! Hypotheses are derived from the engine's own classes: every productive
//! class is probed with placeholder stems to learn its ending inventory, gap
//! surfaces are segmented against that inventory, and each (class, stem)
//! hypothesis is verified by generating the implicated cells through a
//! caller-supplied `LexemeSpec`. Ranking only orders the work; the oracle
//! (the replay under `docs/SYNODAL_GOLD_ORACLE.md`) is the sole reviewer.
//! Accent facts come only from the gold surfaces themselves: an accent
//! paradigm is accepted iff it reproduces every attested printed cell.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::Instant;

use sha2::{Digest, Sha256};
use synodal_church_slavonic::{
    AccentScope, AdjectiveClass, AdjectiveForm, AdjectiveSpec, Animacy, Aspect, Comparison,
    FiniteTense, Gender, GrammarCell, Inflector, NounDeclension, NounSpec, Number,
    OrthographyProfile, SpecificationSource, VerbConjugation, VerbSpec,
};
use synodal_church_slavonic_core::{
    AdjectiveCell, FiniteVerbCell, FormSet, ImperativeCell, LParticipleCell, NounCell, Person,
    normalize_lookup_accentless,
};

use crate::synodal_gold::{
    self, ParadigmOracleRow, Scope, TokenOracleRow, candidate_cell_keys, committed_gap,
    paradigm_expected_variants, paradigm_lemma, strip_accents, surfaces_match,
};

const HYPOTHESES_RELATIVE: &str = "reports/synodal-gold-hypotheses.tsv";
const REJECTED_RELATIVE: &str = "reports/synodal-gold-rejected-hypotheses.tsv";
const PONOMAR_SOURCE: &str = "ponomar-elizabeth-bible-2026-08-09";
const ALYPY_SOURCE: &str = "alypy-gamanovich-grammar-web-2023";
const PONOMAR_RELATIVE: &str = "data/intermediate/synodal/ponomar-elizabeth-bible-2026-08-09.jsonl";
const ALYPY_RELATIVE: &str = "data/intermediate/synodal/alypy-gamanovich-grammar-web-2023.jsonl";
const ARTIFACT_RELATIVE: &str = "crates/synodal-church-slavonic/generated/registry.dat";
const DICTIONARY_RELATIVE: &str = "crates/synodal-church-slavonic-dictionary/generated/registry.rs";

const HYPOTHESES_HEADER: &str = "rank\tcluster_id\tstatus\tpos\tclass\tgender\taspect\tlemma\tstem\tcells\ttoken_keys\tparadigm_keys\taccent\tevidence\tnote";

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

struct ProposeOptions {
    classes: BTreeSet<String>,
    min_cells: usize,
    top: usize,
    output: String,
}

fn parse_propose_options(
    args: &mut impl Iterator<Item = String>,
) -> Result<ProposeOptions, Box<dyn Error>> {
    let mut options = ProposeOptions {
        classes: BTreeSet::new(),
        min_cells: 2,
        top: usize::MAX,
        output: HYPOTHESES_RELATIVE.to_owned(),
    };
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--only" | "--class" => {
                options
                    .classes
                    .insert(args.next().ok_or("--only requires a gap class")?);
            }
            "--min-cells" => {
                options.min_cells = args
                    .next()
                    .ok_or("--min-cells requires a number")?
                    .parse()?;
            }
            "--top" => options.top = args.next().ok_or("--top requires a number")?.parse()?,
            "--output" => options.output = args.next().ok_or("--output requires a path")?,
            other => return Err(format!("unknown synodal-gold propose option: {other}").into()),
        }
    }
    if options.classes.is_empty() {
        options.classes.insert("unregistered-lemma".to_owned());
    }
    Ok(options)
}

pub(crate) fn propose(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let options = parse_propose_options(args)?;
    let started = Instant::now();
    let proposal = propose_clusters(
        root,
        &options.classes,
        options.min_cells,
        Budget {
            max_candidates: options.top,
            want_fit: usize::MAX,
        },
    )?;
    fs::write(root.join(&options.output), render_hypotheses(&proposal))?;
    print_propose_summary(&proposal, &options.output, started.elapsed().as_secs_f64());
    Ok(())
}

struct AdmitOptions {
    input: String,
    take: Option<usize>,
    oracle: Option<&'static str>,
}

fn parse_admit_options(
    args: &mut impl Iterator<Item = String>,
) -> Result<AdmitOptions, Box<dyn Error>> {
    let mut options = AdmitOptions {
        input: HYPOTHESES_RELATIVE.to_owned(),
        take: None,
        oracle: None,
    };
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--take" => {
                options.take = Some(args.next().ok_or("--take requires a number")?.parse()?)
            }
            "--oracle" => {
                options.oracle = Some(match args.next().as_deref() {
                    Some("token") => "token",
                    Some("paradigm") => "paradigm",
                    _ => return Err("--oracle takes token or paradigm".into()),
                });
            }
            other if !other.starts_with("--") => options.input = other.to_owned(),
            other => return Err(format!("unknown synodal-gold admit option: {other}").into()),
        }
    }
    Ok(options)
}

pub(crate) fn admit(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let options = parse_admit_options(args)?;
    let started = Instant::now();
    let content = fs::read_to_string(root.join(&options.input))
        .map_err(|error| format!("read {}: {error}", options.input))?;
    let mut clusters = parse_hypotheses(&content)?;
    clusters.retain(|cluster| cluster.status == ClusterStatus::Fit);
    if let Some(oracle) = options.oracle {
        clusters.retain(|cluster| match oracle {
            "token" => !cluster.token_keys.is_empty(),
            _ => !cluster.paradigm_keys.is_empty(),
        });
    }
    if let Some(take) = options.take {
        clusters.truncate(take);
    }
    let outcome = admit_batch(root, &clusters)?;
    print_admit_summary(&outcome, started.elapsed().as_secs_f64());
    Ok(())
}

/// One line: propose → admit → scoped replay.
pub(crate) fn inner_loop(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let mut classes = BTreeSet::new();
    let mut take = 200usize;
    let mut min_cells = 2usize;
    let mut top: Option<usize> = None;
    let mut oracle: Option<&'static str> = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--only" | "--class" => {
                classes.insert(args.next().ok_or("--only requires a gap class")?);
            }
            "--take" => take = args.next().ok_or("--take requires a number")?.parse()?,
            "--min-cells" => {
                min_cells = args
                    .next()
                    .ok_or("--min-cells requires a number")?
                    .parse()?
            }
            "--top" => top = Some(args.next().ok_or("--top requires a number")?.parse()?),
            "--oracle" => {
                oracle = Some(match args.next().as_deref() {
                    Some("token") => "token",
                    Some("paradigm") => "paradigm",
                    _ => return Err("--oracle takes token or paradigm".into()),
                });
            }
            other => return Err(format!("unknown synodal-gold loop option: {other}").into()),
        }
    }
    if classes.is_empty() {
        classes.insert("unregistered-lemma".to_owned());
    }
    // Verification stops once one batch of fit clusters is in hand: the loop
    // keeps the inner iteration short and re-proposes after every batch.
    let proposal = propose_clusters(
        root,
        &classes,
        min_cells,
        Budget {
            max_candidates: top.unwrap_or(usize::MAX),
            want_fit: take,
        },
    )?;
    fs::write(root.join(HYPOTHESES_RELATIVE), render_hypotheses(&proposal))?;
    let mut clusters = proposal.clusters;
    clusters.retain(|cluster| cluster.status == ClusterStatus::Fit);
    if let Some(oracle) = oracle {
        clusters.retain(|cluster| match oracle {
            "token" => !cluster.token_keys.is_empty(),
            _ => !cluster.paradigm_keys.is_empty(),
        });
    }
    clusters.truncate(take);
    let outcome = admit_batch(root, &clusters)?;
    let mut cleared: Vec<String> = outcome
        .cleared_by_class
        .iter()
        .map(|((oracle, class), count)| format!("{oracle}/{class} {count}"))
        .collect();
    if cleared.is_empty() {
        cleared.push("none".to_owned());
    }
    println!(
        "synodal-gold loop: cleared {} | landed {} rules / {} residue, rejected {} | {:.1}s",
        cleared.join(", "),
        outcome.kept.len(),
        outcome.residue_rows,
        outcome.rejected.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Development aid: `synodal-gold probe <pos> <class> <gender|aspect|-> <lemma>
/// <stem> [scope|placement|mark ...]` installs the hypothesis as a placeholder
/// lexeme and prints every cell the registry path generates for it.
pub(crate) fn probe(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = args.collect();
    let [pos, class_code, feature, lemma, stem, rules @ ..] = arguments.as_slice() else {
        return Err(
            "probe <pos> <class> <gender|aspect|-> <lemma> <stem> [scope|placement|mark ...]"
                .into(),
        );
    };
    let class = productive_classes()
        .into_iter()
        .find(|class| {
            class.pos == pos
                && class.class == class_code
                && (feature == "-" || class.gender == feature || class.aspect == feature)
        })
        .ok_or("unknown class")?;
    let id = format!("synodal:{}:probe-x-0-0", class.pos);
    let mut builder = ArtifactBuilder::load(root)?;
    builder.lexeme(&id, &class, lemma, stem);
    for rule in rules {
        let parts: Vec<&str> = rule.split('|').collect();
        if parts.len() != 3 {
            return Err(format!("malformed rule {rule}").into());
        }
        builder.accent(
            &id,
            &AccentRuleSpec {
                scope: parts[0].to_owned(),
                placement: parts[1].to_owned(),
                mark: parts[2].to_owned(),
            },
        );
    }
    builder.install()?;
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let lexeme = synodal_church_slavonic::LexemeId::from(id);
    for cell in class_cells(&class) {
        match liturgical.form_by_id(&lexeme, cell) {
            Ok(forms) => {
                let variants: Vec<String> = forms
                    .variants()
                    .iter()
                    .map(|variant| format!("{} ({})", variant.printed, variant.expanded))
                    .collect();
                println!("{}\t{}", cell.key(), variants.join(" / "));
            }
            Err(error) => println!("{}\tERROR {error}", cell.key()),
        }
    }
    Ok(())
}

fn spec_form(
    spec: &Spec,
    inflector: Inflector,
    cell: GrammarCell,
) -> synodal_church_slavonic::Result<FormSet> {
    match (spec, cell) {
        (Spec::Noun(spec), GrammarCell::Noun(cell)) => spec.form_with(inflector, cell),
        (Spec::Adjective(spec), GrammarCell::Adjective(cell)) => spec.form_with(inflector, cell),
        (Spec::Verb(spec), cell) => spec.form_with(inflector, cell),
        _ => Err(synodal_church_slavonic::Error::ContradictoryMetadata {
            reason: "cell does not belong to the hypothesis part of speech".into(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Class inventory and probing
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ClassSpec {
    pos: &'static str,
    class: &'static str,
    gender: &'static str,
    aspect: &'static str,
}

impl ClassSpec {
    fn noun(class: &'static str, gender: &'static str) -> Self {
        Self {
            pos: "noun",
            class,
            gender,
            aspect: "",
        }
    }
    fn adjective(class: &'static str) -> Self {
        Self {
            pos: "adjective",
            class,
            gender: "",
            aspect: "",
        }
    }
    fn verb(class: &'static str, aspect: &'static str) -> Self {
        Self {
            pos: "verb",
            class,
            gender: "",
            aspect,
        }
    }
}

/// The productive classes hypotheses may name. Lexically specific classes
/// (the лорд/день/камень/дщерь shapes) and the closed pronoun, numeral, and
/// determiner tables are not hypothesised: those are reviewed by hand.
fn productive_classes() -> Vec<ClassSpec> {
    let mut classes = Vec::new();
    for (class, genders) in [
        ("first-hard-m", &["masculine"][..]),
        ("first-hard-u-stem-m", &["masculine"]),
        ("first-hard-in-ethnonym-m", &["masculine"]),
        ("first-hard-velar-m", &["masculine"]),
        ("first-mixed-m", &["masculine"]),
        ("first-mixed-ts-m", &["masculine"]),
        ("first-hard-n", &["neuter"]),
        ("first-soft-m", &["masculine"]),
        ("first-soft-agent-tel-m", &["masculine"]),
        ("first-soft-j-m", &["masculine"]),
        ("first-soft-ey-m", &["masculine"]),
        ("first-soft-n", &["neuter"]),
        ("first-soft-ishche-n", &["neuter"]),
        ("first-soft-ie-n", &["neuter"]),
        ("second-hard", &["feminine", "masculine"]),
        ("second-hard-velar", &["feminine", "masculine"]),
        ("second-soft", &["feminine", "masculine"]),
        ("second-mixed", &["feminine", "masculine"]),
        ("second-soft-m-ia", &["masculine"]),
        ("second-soft-f-ia", &["feminine"]),
        ("third-f", &["feminine"]),
        ("third-m", &["masculine"]),
        ("fourth-neuter-en", &["neuter"]),
        ("fourth-neuter-es", &["neuter"]),
        ("fourth-neuter-at", &["neuter"]),
        ("fourth-feminine-ov", &["feminine"]),
    ] {
        for gender in genders {
            classes.push(ClassSpec::noun(class, gender));
        }
    }
    for class in [
        "hard-short",
        "soft-short",
        "velar-short",
        "possessive-hard-short",
        "possessive-soft-short",
        "possessive-j-short",
        "possessive-in",
        "possessive-sk",
        "possessive-ii",
    ] {
        classes.push(ClassSpec::adjective(class));
    }
    for class in ["first-unpalatalized", "first-palatalized", "second"] {
        for aspect in ["imperfective", "perfective"] {
            classes.push(ClassSpec::verb(class, aspect));
        }
    }
    classes
}

fn noun_declension(code: &str) -> Option<NounDeclension> {
    Some(match code {
        "first-hard-m" => NounDeclension::FirstHardMasculine,
        "first-hard-u-stem-m" => NounDeclension::FirstHardMasculineUStem,
        "first-hard-in-ethnonym-m" => NounDeclension::FirstHardMasculineInEthnonym,
        "first-hard-ud-es-m" => NounDeclension::FirstHardMasculineUdEs,
        "first-hard-velar-m" => NounDeclension::FirstHardVelarMasculine,
        "first-mixed-m" => NounDeclension::FirstMixedMasculine,
        "first-mixed-ts-m" => NounDeclension::FirstMixedTsMasculine,
        "first-hard-n" => NounDeclension::FirstHardNeuter,
        "first-soft-m" => NounDeclension::FirstSoftMasculine,
        "first-soft-agent-tel-m" => NounDeclension::FirstSoftMasculineAgentTel,
        "first-soft-lord-m" => NounDeclension::FirstSoftMasculineLord,
        "first-soft-j-m" => NounDeclension::FirstSoftMasculineJ,
        "first-soft-ey-m" => NounDeclension::FirstSoftMasculineEy,
        "first-soft-n" => NounDeclension::FirstSoftNeuter,
        "first-soft-ishche-n" => NounDeclension::FirstSoftNeuterIshche,
        "first-soft-ie-n" => NounDeclension::FirstSoftNeuterIe,
        "second-hard" => NounDeclension::SecondHard,
        "second-hard-velar" => NounDeclension::SecondHardVelar,
        "second-soft" => NounDeclension::SecondSoft,
        "second-soft-postvocalic-ancient-pl" => NounDeclension::SecondSoftPostvocalicAncientPlural,
        "second-soft-m-ia" => NounDeclension::SecondSoftMasculineIa,
        "second-soft-f-ia" => NounDeclension::SecondSoftFeminineIa,
        "second-mixed" => NounDeclension::SecondMixed,
        "third-f" => NounDeclension::ThirdFeminine,
        "third-m" => NounDeclension::ThirdMasculine,
        "fourth-neuter-en" => NounDeclension::FourthNeuterEn,
        "fourth-neuter-es" => NounDeclension::FourthNeuterEs,
        "fourth-neuter-es-alt-first" => NounDeclension::FourthNeuterEsAlternatingFirst,
        "fourth-neuter-es-paired-dual" => NounDeclension::FourthNeuterEsPairedDual,
        "fourth-neuter-at" => NounDeclension::FourthNeuterAt,
        "fourth-feminine-er" => NounDeclension::FourthFeminineEr,
        "fourth-feminine-er-daughter" => NounDeclension::FourthFeminineErDaughter,
        "fourth-feminine-ov" => NounDeclension::FourthFeminineOv,
        "fourth-feminine-ov-syncopating" => NounDeclension::FourthFeminineOvSyncopating,
        "fourth-masculine-en" => NounDeclension::FourthMasculineEn,
        "fourth-masculine-en-day" => NounDeclension::FourthMasculineEnDay,
        "fourth-masculine-en-kamen" => NounDeclension::FourthMasculineEnKamen,
        "indeclinable" => NounDeclension::Indeclinable,
        _ => return None,
    })
}

fn adjective_class(code: &str) -> Option<AdjectiveClass> {
    Some(match code {
        "hard-short" => AdjectiveClass::Hard,
        "soft-short" => AdjectiveClass::Soft,
        "velar-short" => AdjectiveClass::Velar,
        "possessive-hard-short" => AdjectiveClass::PossessiveHard,
        "possessive-soft-short" => AdjectiveClass::PossessiveSoft,
        "possessive-j-short" => AdjectiveClass::PossessiveJ,
        "possessive-in" => AdjectiveClass::PossessiveIn,
        "possessive-sk" => AdjectiveClass::PossessiveSk,
        "possessive-ii" => AdjectiveClass::PossessiveIi,
        _ => return None,
    })
}

fn verb_conjugation(code: &str) -> Option<VerbConjugation> {
    Some(match code {
        "first-unpalatalized" => VerbConjugation::FirstUnpalatalized,
        "first-palatalized" => VerbConjugation::FirstPalatalized,
        "second" => VerbConjugation::Second,
        "archaic" => VerbConjugation::Archaic,
        _ => return None,
    })
}

fn gender(code: &str) -> Option<Gender> {
    Some(match code {
        "masculine" => Gender::Masculine,
        "feminine" => Gender::Feminine,
        "neuter" => Gender::Neuter,
        _ => return None,
    })
}

fn aspect(code: &str) -> Aspect {
    match code {
        "imperfective" => Aspect::Imperfective,
        "perfective" => Aspect::Perfective,
        "biaspectual" => Aspect::Biaspectual,
        _ => Aspect::Unknown,
    }
}

/// Positional-letter fold on top of the accentless lookup key: the Synodal
/// presentation letters (`ѡ`/`ѻ`, `є`, `ї`/`і`, `ꙗ`, `ꙋ`, `ѕ`) collapse so a
/// probed ending matches a surface regardless of presentation. Used only to
/// find candidates; every acceptance is exact (contract §3). One char maps
/// to one char, so indices into the unfolded key stay valid.
fn loose_key(value: &str) -> Vec<char> {
    normalize_lookup_accentless(value)
        .chars()
        .map(|character| match character {
            'ѡ' | 'ѻ' | 'ѽ' => 'о',
            'є' => 'е',
            'ї' | 'і' | 'ѵ' => 'и',
            'ꙗ' => 'ѧ',
            'ꙋ' => 'у',
            'ѕ' => 'з',
            other => other,
        })
        .collect()
}

/// The letters a hypothesis stem is spelled with: lowercase, accents and
/// breathings removed, the printed uk digraph half folded to `о` — but the
/// broad on `ѻ` kept, because the registry spells ѻ-initial lemmas with it
/// and the engine's initial presentation reproduces it from the stem. (The
/// lookup projections fold ѻ, so they cannot supply stem letters.)
fn surface_letters(value: &str) -> Vec<char> {
    use unicode_normalization::UnicodeNormalization;
    value
        .chars()
        .map(|character| {
            if character == '\u{1c82}' {
                'о'
            } else {
                character
            }
        })
        .flat_map(char::to_lowercase)
        .flat_map(|character| match character {
            'ѹ' => vec!['о', 'у'],
            other => vec![other],
        })
        .collect::<String>()
        .nfd()
        .filter(|character| {
            !matches!(
                character,
                '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}'
            )
        })
        .nfc()
        .collect()
}

fn loose_match(expected: &str, generated: &str) -> bool {
    loose_key(expected) == loose_key(generated)
}

/// A caller-supplied specification for one hypothesis: identity and class,
/// optionally with the accent paradigm under test. Never an attestation.
enum Spec {
    Noun(NounSpec),
    Adjective(AdjectiveSpec),
    Verb(Box<VerbSpec>),
}

fn spec_source() -> Result<SpecificationSource, Box<dyn Error>> {
    Ok(SpecificationSource::new(
        "gold-hypothesis",
        PONOMAR_SOURCE,
        "synodal-gold propose hypothesis (unreviewed)",
    )?)
}

/// Builds the caller-supplied specification for segmentation and lemma
/// derivation (expanded profile only; acceptance goes through the registry).
fn build_spec(class: &ClassSpec, lemma: &str, stem: &str) -> Result<Spec, Box<dyn Error>> {
    let source = spec_source()?;
    Ok(match class.pos {
        "noun" => {
            let declension = noun_declension(class.class).ok_or("unknown noun class")?;
            let gender = gender(class.gender).ok_or("unknown gender")?;
            Spec::Noun(NounSpec::new(lemma, stem, gender, declension, source)?)
        }
        "adjective" => {
            let class_code = adjective_class(class.class).ok_or("unknown adjective class")?;
            Spec::Adjective(AdjectiveSpec::new(lemma, stem, class_code, source)?)
        }
        "verb" => {
            let conjugation = verb_conjugation(class.class).ok_or("unknown verb class")?;
            let mut builder = VerbSpec::builder(lemma, aspect(class.aspect), conjugation, source)?;
            if !stem.is_empty() {
                builder = builder.present_stem(stem)?;
            }
            Spec::Verb(Box::new(builder.build()?))
        }
        other => return Err(format!("no specification path for part of speech {other}").into()),
    })
}

fn generate(spec: &Spec, inflector: Inflector, cell: GrammarCell) -> Option<FormSet> {
    spec_form(spec, inflector, cell).ok()
}

/// The cells a class can realise (the probe generates all of them).
fn class_cells(class: &ClassSpec) -> Vec<GrammarCell> {
    match class.pos {
        "noun" => NounCell::inventory(&[Animacy::Inanimate, Animacy::Animate])
            .into_iter()
            .map(GrammarCell::Noun)
            .collect(),
        "adjective" => AdjectiveCell::inventory(
            &[AdjectiveForm::Long, AdjectiveForm::Short],
            &[Comparison::Positive],
            |_| &Animacy::ALL,
        )
        .into_iter()
        .map(GrammarCell::Adjective)
        .collect(),
        "verb" => {
            let mut cells: Vec<GrammarCell> = FiniteVerbCell::inventory(&[
                FiniteTense::Present,
                FiniteTense::Future,
                FiniteTense::Aorist,
                FiniteTense::Imperfect,
            ])
            .into_iter()
            .map(GrammarCell::FiniteVerb)
            .collect();
            for number in Number::ALL {
                for person in Person::ALL {
                    cells.push(GrammarCell::Imperative(ImperativeCell { person, number }));
                }
                for gender in Gender::ALL {
                    cells.push(GrammarCell::LParticiple(LParticipleCell { gender, number }));
                }
            }
            cells.push(GrammarCell::Infinitive);
            cells
        }
        _ => Vec::new(),
    }
}

/// One licensed ending of a class, learned by probing: the surface tail that
/// follows the (possibly mutated) stem, the stem characters the mutation
/// consumed, and the cell it realises.
#[derive(Clone, Debug)]
struct Ending {
    class_index: usize,
    cell: GrammarCell,
    /// Stem characters (in lookup normalisation) replaced by the mutation.
    tail: String,
}

/// Probe stems: plain plus each velar and hushing final so mutating classes
/// reveal their alternations. Probes that a class rejects are skipped.
const PROBE_STEMS: &[&str] = &[
    "проб", "прок", "прог", "прох", "проц", "прош", "проч", "прож", "прощ", "прон",
];

fn probe_endings(classes: &[ClassSpec], inflector: Inflector) -> BTreeMap<String, Vec<Ending>> {
    let mut endings: BTreeMap<String, Vec<Ending>> = BTreeMap::new();
    let mut seen: BTreeSet<(String, usize, String, String)> = BTreeSet::new();
    for (class_index, class) in classes.iter().enumerate() {
        for probe in PROBE_STEMS {
            let lemma = match class.pos {
                "verb" => format!("{probe}ити"),
                _ => format!("{probe}ъ"),
            };
            let Ok(spec) = build_spec(class, &lemma, probe) else {
                continue;
            };
            let probe_key: Vec<char> = loose_key(probe);
            for cell in class_cells(class) {
                let Some(forms) = generate(&spec, inflector, cell) else {
                    continue;
                };
                for variant in forms.variants() {
                    for surface in [&variant.printed, &variant.expanded] {
                        let key: Vec<char> = loose_key(surface);
                        let prefix = probe_key
                            .iter()
                            .zip(key.iter())
                            .take_while(|(left, right)| left == right)
                            .count();
                        let consumed = probe_key.len() - prefix;
                        if consumed > 2 || prefix < 2 {
                            continue;
                        }
                        let ending: String = key[prefix..].iter().collect();
                        let tail: String = probe_key[prefix..].iter().collect();
                        let cell_key = cell.key();
                        if !seen.insert((ending.clone(), class_index, cell_key, tail.clone())) {
                            continue;
                        }
                        endings.entry(ending).or_default().push(Ending {
                            class_index,
                            cell,
                            tail,
                        });
                    }
                }
            }
        }
    }
    endings
}

// ---------------------------------------------------------------------------
// Hypotheses and clusters
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClusterStatus {
    /// Class and accent paradigm reproduce every attested cell.
    Fit,
    /// Below the `--min-cells` floor: listed, never admitted automatically.
    BelowMinCells,
    /// The class reproduces the cells accentlessly, but no reviewed accent
    /// placement grammar reproduces all printed accents.
    AccentUnfit,
    /// The class does not reproduce the cells even accentlessly.
    FormUnfit,
    /// The lemma already resolves to a registered lexeme.
    LemmaRegistered,
    /// A higher-ranked hypothesis in the same run claims the same lemma.
    LemmaCompeting,
    /// Another class of the same lemma reproduces strictly more attested
    /// cells accentlessly: this class explains a subset and is never forced.
    ClassSubsumed,
    /// A higher-ranked fit hypothesis in the same run already claims one of
    /// its attested cells; re-proposed once those cells have cleared.
    CellsCompeting,
    /// Above the verification budget (`--top`): listed unverified.
    Unverified,
    /// The class reproduces only part of the Alypy table: a subclass or a
    /// genuine irregular is needed, never a forced partial admission.
    Partial,
    /// The class reproduces its cluster but would also analyse other gap
    /// surfaces it cannot reproduce exactly, moving them to an engine class.
    Collateral,
}

impl ClusterStatus {
    fn code(self) -> &'static str {
        match self {
            Self::Fit => "fit",
            Self::BelowMinCells => "below-min-cells",
            Self::AccentUnfit => "accent-unfit",
            Self::FormUnfit => "form-unfit",
            Self::LemmaRegistered => "lemma-registered",
            Self::LemmaCompeting => "lemma-competing",
            Self::ClassSubsumed => "class-subsumed",
            Self::CellsCompeting => "cells-competing",
            Self::Unverified => "unverified",
            Self::Partial => "partial",
            Self::Collateral => "collateral-unfit",
        }
    }

    fn parse(code: &str) -> Option<Self> {
        Some(match code {
            "fit" => Self::Fit,
            "below-min-cells" => Self::BelowMinCells,
            "accent-unfit" => Self::AccentUnfit,
            "form-unfit" => Self::FormUnfit,
            "lemma-registered" => Self::LemmaRegistered,
            "lemma-competing" => Self::LemmaCompeting,
            "class-subsumed" => Self::ClassSubsumed,
            "cells-competing" => Self::CellsCompeting,
            "unverified" => Self::Unverified,
            "partial" => Self::Partial,
            "collateral-unfit" => Self::Collateral,
            _ => return None,
        })
    }
}

/// One attested gap cell the hypothesis must reproduce: a token surface with
/// its candidate cells, or a paradigm-oracle row with its printed variants.
#[derive(Clone, Debug)]
struct AttestedCell {
    oracle: &'static str,
    key: String,
    /// Accepted printed surfaces (one for a token; the Alypy variants for a
    /// paradigm cell).
    expected: Vec<String>,
    cells: Vec<GrammarCell>,
    /// Passage references (token) or the Alypy section (paradigm).
    evidence: Vec<String>,
}

#[derive(Clone, Debug)]
struct AccentRuleSpec {
    scope: String,
    placement: String,
    mark: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Cluster {
    id: String,
    status: ClusterStatus,
    class: ClassSpec,
    lemma: String,
    stem: String,
    cells: Vec<AttestedCell>,
    token_keys: Vec<String>,
    paradigm_keys: Vec<String>,
    accent: Vec<AccentRuleSpec>,
    evidence: Vec<String>,
    note: String,
}

fn cluster_id(class: &ClassSpec, lemma: &str, stem: &str) -> String {
    let digest = Sha256::digest(
        format!(
            "{}|{}|{}|{}|{lemma}|{stem}",
            class.pos, class.class, class.gender, class.aspect
        )
        .as_bytes(),
    );
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("gold-{}", &hex[..12])
}

/// A hypothesis awaiting verification through the registry path.
struct Candidate {
    class_index: usize,
    lemma: String,
    stem: String,
    cells: Vec<AttestedCell>,
    status: ClusterStatus,
    accent: Vec<AccentRuleSpec>,
    note: String,
    /// The closest fixed accent rule and the cells it misses (for the
    /// accent-unfit note).
    diagnostic: String,
}

/// How much verification a proposal run does: at most `max_candidates`
/// candidates (largest clusters first), stopping early once `want_fit` fit
/// clusters have been found (the loop needs one batch, not the whole gap).
#[derive(Clone, Copy, Debug)]
struct Budget {
    max_candidates: usize,
    want_fit: usize,
}

/// The proposal: ranked clusters plus the count of segmentation hypotheses
/// below the `--min-cells` floor.
struct Proposal {
    clusters: Vec<Cluster>,
    below_floor: usize,
}

fn propose_clusters(
    root: &Path,
    classes_in_scope: &BTreeSet<String>,
    min_cells: usize,
    budget: Budget,
) -> Result<Proposal, Box<dyn Error>> {
    let committed = committed_gap(root)?;
    let token_rows = synodal_gold::load_token_oracle(root)?;
    let paradigm_rows = synodal_gold::load_paradigm_oracle(root)?;
    // Segmentation and lemma derivation run through caller-supplied
    // specifications in the expanded profile (the liturgical profile refuses
    // a specification without accent metadata); every acceptance below goes
    // through the registry path the gate itself uses.
    let expanded = Inflector::builder()
        .orthography(OrthographyProfile::Expanded)
        .build();
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let classes = productive_classes();
    let endings = probe_endings(&classes, expanded);

    // Token-side hypotheses by segmentation against the probed endings.
    let mut hypotheses: BTreeMap<(usize, String), Vec<AttestedCell>> = BTreeMap::new();
    let token_by_surface: BTreeMap<&str, &TokenOracleRow> = token_rows
        .iter()
        .map(|row| (row.surface.as_str(), row))
        .collect();
    for ((oracle, key), reason) in &committed {
        if oracle != "token" || !classes_in_scope.contains(reason) {
            continue;
        }
        let Some(row) = token_by_surface.get(key.as_str()) else {
            continue;
        };
        if !row.non_lexical.is_empty() {
            continue;
        }
        let surface_key = loose_key(&row.surface);
        let surface_letters = surface_letters(&row.surface);
        if surface_letters.len() != surface_key.len() {
            continue;
        }
        let mut per_hypothesis: BTreeMap<(usize, String), Vec<GrammarCell>> = BTreeMap::new();
        for ending_length in 0..=6usize {
            if surface_key.len() < ending_length + 2 {
                break;
            }
            let split = surface_key.len() - ending_length;
            let ending: String = surface_key[split..].iter().collect();
            let Some(candidates) = endings.get(&ending) else {
                continue;
            };
            let head: String = surface_letters[..split].iter().collect();
            for candidate in candidates {
                let stem = format!("{head}{}", candidate.tail);
                per_hypothesis
                    .entry((candidate.class_index, stem))
                    .or_default()
                    .push(candidate.cell);
            }
        }
        for ((class_index, stem), mut cells) in per_hypothesis {
            cells.sort_by_key(|cell| cell.key());
            cells.dedup();
            hypotheses
                .entry((class_index, stem))
                .or_default()
                .push(AttestedCell {
                    oracle: "token",
                    key: row.surface.clone(),
                    expected: vec![row.surface.clone()],
                    cells,
                    evidence: row.references.iter().take(3).cloned().collect(),
                });
        }
    }

    // Paradigm-side hypotheses: the Alypy headword is the lemma; every class
    // of the row's part of speech is a candidate for the whole table.
    let mut tables: BTreeMap<(String, String), Vec<&ParadigmOracleRow>> = BTreeMap::new();
    for row in &paradigm_rows {
        if !committed
            .get(&("paradigm".to_owned(), row.key.clone()))
            .is_some_and(|reason| classes_in_scope.contains(reason))
        {
            continue;
        }
        tables
            .entry((row.section.clone(), row.headword.clone()))
            .or_default()
            .push(row);
    }
    let mut candidates: Vec<Candidate> = Vec::new();
    for ((section, headword), rows) in &tables {
        let Some(lemma) = paradigm_lemma(headword) else {
            continue;
        };
        let lemma_key = loose_key(&lemma);
        let lemma_letters = surface_letters(&lemma);
        if lemma_letters.len() != lemma_key.len() {
            continue;
        }
        let pos = rows[0].pos.as_str();
        let cells: Vec<AttestedCell> = rows
            .iter()
            .map(|row| AttestedCell {
                oracle: "paradigm",
                key: row.key.clone(),
                expected: paradigm_expected_variants(&row.surface),
                cells: candidate_cell_keys(row)
                    .iter()
                    .filter_map(|key| key.parse::<GrammarCell>().ok())
                    .collect(),
                evidence: vec![section.clone()],
            })
            .collect();
        for (class_index, class) in classes.iter().enumerate() {
            if class.pos != pos {
                continue;
            }
            // Stems: the headword minus a nominative-singular (noun), masculine
            // nominative-singular (adjective), or infinitive (verb) ending.
            let mut stems: BTreeSet<String> = BTreeSet::new();
            for ending_length in 0..=4usize {
                if lemma_key.len() < ending_length + 2 {
                    break;
                }
                let split = lemma_key.len() - ending_length;
                let ending: String = lemma_key[split..].iter().collect();
                let Some(matches) = endings.get(&ending) else {
                    continue;
                };
                for candidate in matches {
                    if candidate.class_index != class_index || !is_lemma_cell(candidate.cell) {
                        continue;
                    }
                    let head: String = lemma_letters[..split].iter().collect();
                    stems.insert(format!("{head}{}", candidate.tail));
                }
            }
            if class.pos == "verb" {
                stems.insert(String::new());
            }
            for stem in stems {
                candidates.push(Candidate {
                    class_index,
                    lemma: strip_accents(&lemma),
                    stem,
                    cells: cells.clone(),
                    status: ClusterStatus::FormUnfit,
                    accent: Vec::new(),
                    note: String::new(),
                    diagnostic: String::new(),
                });
            }
        }
    }

    let registered: BTreeSet<String> = synodal_church_slavonic::lexemes()?
        .iter()
        .map(|lexeme| normalize_lookup_accentless(lexeme.lemma()))
        .collect();

    let mut clusters: Vec<Cluster> = Vec::new();
    let mut below_floor = 0usize;
    for ((class_index, stem), cells) in hypotheses {
        let class = &classes[class_index];
        if cells.len() < min_cells {
            // Not verified in this run: the long tail below the floor is
            // worked after the multi-cell clusters (rerun with a lower
            // --min-cells). Counted, not listed, so the report stays small.
            below_floor += 1;
            continue;
        }
        let Ok(spec) = build_spec(class, &format!("{stem}ъ"), &stem) else {
            continue;
        };
        let Some(lemma) = hypothesis_lemma(&spec, class, expanded) else {
            continue;
        };
        candidates.push(Candidate {
            class_index,
            lemma,
            stem,
            cells,
            status: ClusterStatus::FormUnfit,
            accent: Vec::new(),
            note: String::new(),
            diagnostic: String::new(),
        });
    }

    // Every token gap surface by loose key, for the collateral check.
    let mut gap_surfaces: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (oracle, key) in committed.keys() {
        if oracle == "token" {
            gap_surfaces
                .entry(loose_key(key).into_iter().collect())
                .or_default()
                .push(key.clone());
        }
    }
    verify_candidates(
        root,
        &classes,
        &mut candidates,
        expanded,
        liturgical,
        &registered,
        &gap_surfaces,
        budget,
    )?;

    for candidate in candidates {
        clusters.push(candidate.into_cluster(&classes));
    }

    // Rank: attested cells cleared per admission, fit clusters first.
    clusters.sort_by(|left, right| {
        let rank = |cluster: &Cluster| {
            (
                matches!(cluster.status, ClusterStatus::Fit),
                cluster.cells.len(),
            )
        };
        rank(right)
            .cmp(&rank(left))
            .then_with(|| left.lemma.cmp(&right.lemma))
            .then_with(|| left.class.class.cmp(right.class.class))
    });
    // One admission per lemma and one claimant per attested cell per run: a
    // fit cluster whose lemma or cells a higher-ranked fit cluster already
    // claims is marked competing (an ambiguous class, or a rival lemma for the
    // same surfaces) and is re-proposed after the claimant lands.
    let mut claimed_lemmas: BTreeSet<String> = BTreeSet::new();
    let mut claimed_cells: BTreeSet<(&'static str, String)> = BTreeSet::new();
    for cluster in &mut clusters {
        if cluster.status != ClusterStatus::Fit {
            continue;
        }
        if !claimed_lemmas.insert(normalize_lookup_accentless(&cluster.lemma)) {
            cluster.status = ClusterStatus::LemmaCompeting;
            continue;
        }
        if cluster
            .cells
            .iter()
            .any(|cell| claimed_cells.contains(&(cell.oracle, cell.key.clone())))
        {
            cluster.status = ClusterStatus::CellsCompeting;
            continue;
        }
        claimed_cells.extend(
            cluster
                .cells
                .iter()
                .map(|cell| (cell.oracle, cell.key.clone())),
        );
    }
    Ok(Proposal {
        clusters,
        below_floor,
    })
}

impl Candidate {
    fn into_cluster(self, classes: &[ClassSpec]) -> Cluster {
        let class = classes[self.class_index].clone();
        let id = cluster_id(&class, &self.lemma, &self.stem);
        let token_keys = self
            .cells
            .iter()
            .filter(|cell| cell.oracle == "token")
            .map(|cell| cell.key.clone())
            .collect();
        let paradigm_keys = self
            .cells
            .iter()
            .filter(|cell| cell.oracle == "paradigm")
            .map(|cell| cell.key.clone())
            .collect();
        let mut evidence: Vec<String> = self
            .cells
            .iter()
            .flat_map(|cell| cell.evidence.clone())
            .collect();
        evidence.sort();
        evidence.dedup();
        Cluster {
            id,
            status: self.status,
            class,
            lemma: self.lemma,
            stem: self.stem,
            cells: self.cells,
            token_keys,
            paradigm_keys,
            accent: self.accent,
            evidence,
            note: self.note,
        }
    }
}

fn is_lemma_cell(cell: GrammarCell) -> bool {
    match cell {
        GrammarCell::Noun(cell) => {
            cell.case == synodal_church_slavonic::Case::Nominative
                && cell.number == Number::Singular
        }
        GrammarCell::Adjective(cell) => {
            cell.case == synodal_church_slavonic::Case::Nominative
                && cell.number == Number::Singular
                && cell.gender == Gender::Masculine
        }
        GrammarCell::Infinitive => true,
        _ => false,
    }
}

/// The lemma a hypothesis would register: the generated nominative singular
/// (masculine, long form for adjectives), accentless, in the expanded
/// orthography the registry uses for lemmas.
fn hypothesis_lemma(spec: &Spec, class: &ClassSpec, inflector: Inflector) -> Option<String> {
    let cell = match class.pos {
        "noun" => GrammarCell::Noun(NounCell {
            case: synodal_church_slavonic::Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        }),
        "adjective" => GrammarCell::Adjective(AdjectiveCell {
            case: synodal_church_slavonic::Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Long,
            comparison: Comparison::Positive,
        }),
        _ => return None,
    };
    let forms = generate(spec, inflector, cell)?;
    let variant = forms.variants().first()?;
    Some(strip_accents(&variant.expanded))
}

// ---------------------------------------------------------------------------
// Verification through the registry path
// ---------------------------------------------------------------------------

/// A registry artifact under construction: the committed artifact plus
/// placeholder rows, installed in-process so `form_by_id` generates a
/// hypothesis exactly as the gate would generate an admitted lexeme
/// (positional presentation, breathings, and accent paradigm included).
struct ArtifactBuilder {
    lines: Vec<String>,
    lexemes: Vec<String>,
    accents: Vec<String>,
    evidence: Vec<String>,
}

const PROBE_EVIDENCE: &str = "gold-probe";

impl ArtifactBuilder {
    fn load(root: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(root.join(ARTIFACT_RELATIVE))?;
        Ok(Self {
            lines: content.lines().map(str::to_owned).collect(),
            lexemes: Vec::new(),
            accents: Vec::new(),
            evidence: vec![format!(
                "{PROBE_EVIDENCE}\t{PONOMAR_SOURCE}\tsynodal-russian\tsynodal-gold propose\ttarget-attestation\tunreviewed hypothesis probe; never written to the curated data"
            )],
        })
    }

    fn lexeme(&mut self, id: &str, class: &ClassSpec, lemma: &str, stem: &str) {
        self.lexemes.push(format!(
            "{id}\t{lemma}\t{}\t{}\t{stem}\t{}\t{}\t{PONOMAR_SOURCE}\tsynodal-russian",
            class.pos, class.class, class.gender, class.aspect
        ));
    }

    fn accent(&mut self, id: &str, rule: &AccentRuleSpec) {
        self.accents.push(format!(
            "{id}\tsynodal-accent:{id}\t{}\t{}\t{}\t\t{PROBE_EVIDENCE}\t{PONOMAR_SOURCE}\tsynodal-gold propose\tsynodal-russian\tsynodal-russian",
            rule.scope, rule.placement, rule.mark
        ));
    }

    /// Composes the artifact, keeping every keyed table sorted by its first
    /// column (the runtime binary-searches them), and installs it.
    fn install(&self) -> Result<(), Box<dyn Error>> {
        let mut output = String::new();
        let mut current: Option<&str> = None;
        let mut section: Vec<&str> = Vec::new();
        let flush = |output: &mut String, name: Option<&str>, section: &mut Vec<&str>| {
            let extra: &[String] = match name {
                Some("LEXEMES") => &self.lexemes,
                Some("ACCENT_PARADIGMS") => &self.accents,
                Some("REVIEWED_EVIDENCE") => &self.evidence,
                _ => &[],
            };
            let mut rows: Vec<&str> = std::mem::take(section);
            rows.extend(extra.iter().map(String::as_str));
            if !extra.is_empty() {
                rows.sort_by_key(|row| row.split('\t').next().unwrap_or_default());
            }
            for row in rows {
                output.push_str(row);
                output.push('\n');
            }
        };
        for line in &self.lines {
            if let Some(header) = line.strip_prefix('@') {
                flush(&mut output, current, &mut section);
                current = header.split(' ').next();
                output.push_str(line);
                output.push('\n');
            } else if current.is_none() {
                output.push_str(line);
                output.push('\n');
            } else {
                section.push(line);
            }
        }
        flush(&mut output, current, &mut section);
        synodal_church_slavonic::install_registry_override(output)?;
        Ok(())
    }
}

/// The candidate cells of one attested cell that the lexeme `id` reproduces:
/// a generated variant matches an expected surface under contract §3.
/// `loose` is the form filter (accents and presentation letters folded, run
/// in the expanded profile because the liturgical profile refuses a lexeme
/// without accent metadata); acceptance is always exact.
fn cell_reproduced(
    inflector: Inflector,
    id: &synodal_church_slavonic::LexemeId,
    cell: &AttestedCell,
    loose: bool,
) -> Vec<GrammarCell> {
    cell.cells
        .iter()
        .copied()
        .filter(|candidate| {
            inflector.form_by_id(id, *candidate).is_ok_and(|forms| {
                forms.variants().iter().any(|variant| {
                    let outputs: Vec<&String> = if cell.oracle == "paradigm" {
                        vec![&variant.printed, &variant.expanded]
                    } else {
                        vec![&variant.printed]
                    };
                    outputs.iter().any(|output| {
                        cell.expected.iter().any(|expected| {
                            if loose {
                                loose_match(expected, output)
                            } else {
                                surfaces_match(expected, output)
                            }
                        })
                    })
                })
            })
        })
        .collect()
}

fn accent_placements() -> Vec<String> {
    let mut placements: Vec<String> = (0..=4u8)
        .map(|index| format!("stem-vowel-from-start:{index}"))
        .collect();
    placements.extend((0..=1u8).map(|index| format!("ending-vowel-from-end:{index}")));
    placements.extend((0..=4u8).map(|index| format!("word-vowel-from-start:{index}")));
    placements
}

const ACCENT_MARKS: &[&str] = &["acute", "grave", "kamora"];

/// Every fixed-scope rule the search tries, in preference order.
fn fixed_rules() -> Vec<AccentRuleSpec> {
    let mut rules = Vec::new();
    for placement in accent_placements() {
        for mark in ACCENT_MARKS {
            rules.push(AccentRuleSpec {
                scope: "all".into(),
                placement: placement.clone(),
                mark: (*mark).to_owned(),
            });
        }
    }
    rules
}

/// The accent scope groups a class's cells fall into: one rule per group is
/// the most a hypothesis may carry (fixed stress first, then per-number
/// mobility). Anything finer is memorisation, not a paradigm.
fn scope_groups(class: &ClassSpec) -> Vec<(String, AccentScope)> {
    let numbers = [
        ("singular", Number::Singular),
        ("dual", Number::Dual),
        ("plural", Number::Plural),
    ];
    let mut groups = Vec::new();
    match class.pos {
        "noun" => {
            for (name, number) in numbers {
                groups.push((
                    format!("noun:{name}"),
                    AccentScope::Noun {
                        numbers: vec![number],
                    },
                ));
            }
        }
        "adjective" => {
            for (form_name, form) in [
                ("long", AdjectiveForm::Long),
                ("short", AdjectiveForm::Short),
            ] {
                for (name, number) in numbers {
                    groups.push((
                        format!("adjective:{form_name}:positive:{name}"),
                        AccentScope::Adjective {
                            form,
                            comparison: Comparison::Positive,
                            numbers: vec![number],
                        },
                    ));
                }
            }
        }
        "verb" => {
            for (tense_name, tense) in [
                ("present", FiniteTense::Present),
                ("future", FiniteTense::Future),
                ("aorist", FiniteTense::Aorist),
                ("imperfect", FiniteTense::Imperfect),
            ] {
                for (name, number) in numbers {
                    groups.push((
                        format!("finite:{tense_name}:{name}"),
                        AccentScope::FiniteVerb {
                            tense,
                            numbers: vec![number],
                        },
                    ));
                }
            }
            for (name, number) in numbers {
                groups.push((
                    format!("imperative:{name}"),
                    AccentScope::Imperative {
                        numbers: vec![number],
                    },
                ));
                groups.push((
                    format!("l-participle:{name}"),
                    AccentScope::LParticiple {
                        numbers: vec![number],
                    },
                ));
            }
        }
        _ => {}
    }
    groups
}

const NOUN_CASES: &[(&str, synodal_church_slavonic::Case)] = &[
    ("nominative", synodal_church_slavonic::Case::Nominative),
    ("genitive", synodal_church_slavonic::Case::Genitive),
    ("dative", synodal_church_slavonic::Case::Dative),
    ("accusative", synodal_church_slavonic::Case::Accusative),
    ("instrumental", synodal_church_slavonic::Case::Instrumental),
    ("locative", synodal_church_slavonic::Case::Locative),
    ("vocative", synodal_church_slavonic::Case::Vocative),
];

/// The rules a candidate's per-number search settled plus the number groups
/// it could not fit with one rule.
type UnfitGroups = (Vec<AccentRuleSpec>, Vec<(String, AccentScope)>);

/// Failing rows attributed to the admitted lexemes that reach them.
type Attribution = BTreeMap<(String, String), BTreeSet<String>>;

/// The attested cells (restricted to their candidate cells) a scope covers.
fn scoped_cells(cells: &[AttestedCell], scope: &AccentScope) -> Vec<AttestedCell> {
    cells
        .iter()
        .filter_map(|cell| {
            let cells: Vec<GrammarCell> = cell
                .cells
                .iter()
                .copied()
                .filter(|candidate| scope.applies_to(*candidate))
                .collect();
            (!cells.is_empty()).then(|| AttestedCell {
                cells,
                ..cell.clone()
            })
        })
        .collect()
}

/// Runs the verification phases through the registry path:
/// A. accentless form reproduction (drops segmentation coincidences);
/// B. one fixed accent rule over every cell;
/// C. one rule per scope group, for clusters B could not fit;
/// D. confirmation of the combined per-group rules.
#[allow(clippy::too_many_arguments)]
fn verify_candidates(
    root: &Path,
    classes: &[ClassSpec],
    candidates: &mut [Candidate],
    expanded: Inflector,
    liturgical: Inflector,
    registered: &BTreeSet<String>,
    gap_surfaces: &BTreeMap<String, Vec<String>>,
    budget: Budget,
) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let id = |phase: &str, index: usize, variant: usize, class: &ClassSpec| {
        synodal_church_slavonic::LexemeId::from(format!(
            "synodal:{}:probe-{phase}-{index}-{variant}",
            class.pos
        ))
    };
    // Phase A.
    let mut builder = ArtifactBuilder::load(root)?;
    for (index, candidate) in candidates.iter().enumerate() {
        let class = &classes[candidate.class_index];
        builder.lexeme(
            id("a", index, 0, class).as_str(),
            class,
            &candidate.lemma,
            &candidate.stem,
        );
    }
    builder.install()?;
    for (index, candidate) in candidates.iter_mut().enumerate() {
        let class = &classes[candidate.class_index];
        let lexeme = id("a", index, 0, class);
        let mut reproduced = Vec::new();
        let table_cells = candidate.cells.len();
        let is_table = candidate.cells.iter().all(|cell| cell.oracle == "paradigm");
        for mut cell in candidate.cells.drain(..) {
            cell.cells = cell_reproduced(expanded, &lexeme, &cell, true);
            if !cell.cells.is_empty() {
                reproduced.push(cell);
            }
        }
        candidate.cells = reproduced;
        if candidate.cells.is_empty() {
            candidate.status = ClusterStatus::FormUnfit;
            candidate.note = "no attested cell reproduced accentlessly".into();
        } else if is_table && candidate.cells.len() < table_cells {
            candidate.status = ClusterStatus::Partial;
            candidate.note = format!(
                "reproduces {} of {table_cells} table cells accentlessly",
                candidate.cells.len()
            );
        } else if registered.contains(&normalize_lookup_accentless(&candidate.lemma)) {
            candidate.status = ClusterStatus::LemmaRegistered;
        } else {
            candidate.status = ClusterStatus::AccentUnfit;
        }
    }
    // Subsumption: among the classes hypothesised for one lemma, only those
    // reproducing the most cells accentlessly go on; a class explaining a
    // strict subset would be a forced partial fit.
    let mut best_by_lemma: BTreeMap<String, usize> = BTreeMap::new();
    for candidate in candidates.iter() {
        if candidate.status == ClusterStatus::AccentUnfit {
            let entry = best_by_lemma
                .entry(normalize_lookup_accentless(&candidate.lemma))
                .or_default();
            *entry = (*entry).max(candidate.cells.len());
        }
    }
    for candidate in candidates.iter_mut() {
        if candidate.status == ClusterStatus::AccentUnfit
            && best_by_lemma[&normalize_lookup_accentless(&candidate.lemma)] > candidate.cells.len()
        {
            candidate.status = ClusterStatus::ClassSubsumed;
        }
    }
    eprintln!(
        "  verify: phase A (accentless form filter) {} candidates, {:.1}s",
        candidates.len(),
        started.elapsed().as_secs_f64()
    );
    // Phases B–E run over windows of the largest pending clusters: each
    // installed artifact is leaked for the process lifetime (rows borrow from
    // it), so the placeholder count per install is bounded, and the loop can
    // stop as soon as one batch of fit clusters exists.
    let fixed = fixed_rules();
    let mut pending: Vec<usize> = (0..candidates.len())
        .filter(|index| candidates[*index].status == ClusterStatus::AccentUnfit)
        .collect();
    pending.sort_by_key(|index| std::cmp::Reverse(candidates[*index].cells.len()));
    for &index in pending.iter().skip(budget.max_candidates) {
        candidates[index].status = ClusterStatus::Unverified;
    }
    pending.truncate(budget.max_candidates);
    let mut fit_found = 0usize;
    let mut verified = 0usize;
    for window in pending.chunks(300) {
        if fit_found >= budget.want_fit {
            for &index in window {
                candidates[index].status = ClusterStatus::Unverified;
            }
            continue;
        }
        verified += window.len();
        verify_window(
            root,
            classes,
            candidates,
            window,
            &fixed,
            liturgical,
            gap_surfaces,
        )?;
        fit_found += window
            .iter()
            .filter(|index| candidates[**index].status == ClusterStatus::Fit)
            .count();
    }
    for candidate in candidates.iter_mut() {
        if candidate.status == ClusterStatus::AccentUnfit && candidate.note.is_empty() {
            candidate.note = if candidate.diagnostic.is_empty() {
                "no placement grammar reproduces every printed accent".into()
            } else {
                format!(
                    "no placement grammar reproduces every printed accent; {}",
                    candidate.diagnostic
                )
            };
        }
    }
    eprintln!(
        "  verify: phases B–E over {verified} candidates, {fit_found} fit, {:.1}s",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Phases B–E for one window of candidates (see `verify_candidates`).
fn verify_window(
    root: &Path,
    classes: &[ClassSpec],
    candidates: &mut [Candidate],
    window: &[usize],
    fixed: &[AccentRuleSpec],
    liturgical: Inflector,
    gap_surfaces: &BTreeMap<String, Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    let id = |phase: &str, index: usize, variant: usize, class: &ClassSpec| {
        synodal_church_slavonic::LexemeId::from(format!(
            "synodal:{}:probe-{phase}-{index}-{variant}",
            class.pos
        ))
    };
    let pending: Vec<usize> = window.to_vec();
    // Phase B.
    for chunk in pending.chunks(600) {
        let mut builder = ArtifactBuilder::load(root)?;
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            for (variant, rule) in fixed.iter().enumerate() {
                let lexeme = id("b", index, variant, class);
                builder.lexeme(lexeme.as_str(), class, &candidate.lemma, &candidate.stem);
                builder.accent(lexeme.as_str(), rule);
            }
        }
        builder.install()?;
        for &index in chunk {
            let candidate = &mut candidates[index];
            let class = &classes[candidate.class_index];
            let mut best: Option<(usize, usize)> = None;
            for (variant, rule) in fixed.iter().enumerate() {
                let lexeme = id("b", index, variant, class);
                // A variant that misses the first cell cannot fit; only
                // variants passing it are counted in full (abandoning one as
                // soon as it can no longer beat the best so far).
                let total = candidate.cells.len();
                let floor = best.map_or(0, |(_, count)| count);
                let mut reproduced = 0usize;
                for (seen, cell) in candidate.cells.iter().enumerate() {
                    if reproduced + (total - seen) <= floor || (seen == 1 && reproduced == 0) {
                        break;
                    }
                    if !cell_reproduced(liturgical, &lexeme, cell, false).is_empty() {
                        reproduced += 1;
                    }
                }
                if reproduced == candidate.cells.len() {
                    candidate.status = ClusterStatus::Fit;
                    candidate.accent = vec![rule.clone()];
                    best = None;
                    break;
                }
                if best.is_none_or(|(_, count)| reproduced > count) {
                    best = Some((variant, reproduced));
                }
            }
            if let Some((variant, reproduced)) = best {
                let lexeme = id("b", index, variant, class);
                let missed: Vec<String> = candidate
                    .cells
                    .iter()
                    .filter(|cell| cell_reproduced(liturgical, &lexeme, cell, false).is_empty())
                    .map(|cell| {
                        let engine = cell
                            .cells
                            .first()
                            .and_then(|candidate| liturgical.form_by_id(&lexeme, *candidate).ok())
                            .and_then(|forms| {
                                forms
                                    .variants()
                                    .first()
                                    .map(|variant| variant.printed.clone())
                            })
                            .unwrap_or_default();
                        format!("{}→{engine}", cell.expected.join("/"))
                    })
                    .take(6)
                    .collect();
                candidate.diagnostic = format!(
                    "closest {}|{}|{} reproduces {reproduced}/{}; misses {}",
                    fixed[variant].scope,
                    fixed[variant].placement,
                    fixed[variant].mark,
                    candidate.cells.len(),
                    missed.join(" ")
                );
            }
        }
    }
    // Phase C.
    let pending: Vec<usize> = pending
        .into_iter()
        .filter(|index| candidates[*index].status == ClusterStatus::AccentUnfit)
        .collect();
    let mut combined: BTreeMap<usize, Vec<AccentRuleSpec>> = BTreeMap::new();
    // Per-number groups a candidate could not fit with one rule; nouns get a
    // per-case refinement (the reviewed data's `noun:<number>:<case>` scopes).
    let mut unfit_groups: BTreeMap<usize, UnfitGroups> = BTreeMap::new();
    for chunk in pending.chunks(100) {
        let mut builder = ArtifactBuilder::load(root)?;
        let mut group_rules: BTreeMap<usize, Vec<AccentRuleSpec>> = BTreeMap::new();
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            let mut rules = Vec::new();
            for (scope_code, _) in scope_groups(class) {
                for rule in fixed {
                    rules.push(AccentRuleSpec {
                        scope: scope_code.clone(),
                        ..rule.clone()
                    });
                }
            }
            for (variant, rule) in rules.iter().enumerate() {
                let lexeme = id("c", index, variant, class);
                builder.lexeme(lexeme.as_str(), class, &candidate.lemma, &candidate.stem);
                builder.accent(lexeme.as_str(), rule);
            }
            group_rules.insert(index, rules);
        }
        builder.install()?;
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            let mut chosen: Vec<AccentRuleSpec> = Vec::new();
            let mut failed: Vec<(String, AccentScope)> = Vec::new();
            for (scope_code, scope) in scope_groups(class) {
                let group = scoped_cells(&candidate.cells, &scope);
                if group.is_empty() {
                    continue;
                }
                let found = group_rules[&index]
                    .iter()
                    .enumerate()
                    .filter(|(_, rule)| rule.scope == scope_code)
                    .find(|(variant, _)| {
                        let lexeme = id("c", index, *variant, class);
                        group.iter().all(|cell| {
                            !cell_reproduced(liturgical, &lexeme, cell, false).is_empty()
                        })
                    });
                match found {
                    Some((_, rule)) => chosen.push(rule.clone()),
                    None => failed.push((scope_code, scope)),
                }
            }
            if failed.is_empty() && !chosen.is_empty() {
                combined.insert(index, chosen);
            } else if class.pos == "noun" && !failed.is_empty() {
                unfit_groups.insert(index, (chosen, failed));
            }
        }
    }
    // Phase C2: per-case refinement of the failed noun number groups.
    let refine: Vec<usize> = unfit_groups.keys().copied().collect();
    for chunk in refine.chunks(40) {
        let mut builder = ArtifactBuilder::load(root)?;
        let mut case_rules: BTreeMap<usize, Vec<(AccentRuleSpec, AccentScope)>> = BTreeMap::new();
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            let mut rules = Vec::new();
            for (number_code, number_scope) in &unfit_groups[&index].1 {
                let AccentScope::Noun { numbers } = number_scope else {
                    continue;
                };
                for (case_code, case) in NOUN_CASES {
                    let scope = AccentScope::NounCases {
                        numbers: numbers.clone(),
                        cases: vec![*case],
                    };
                    for rule in fixed {
                        rules.push((
                            AccentRuleSpec {
                                scope: format!("{number_code}:{case_code}"),
                                ..rule.clone()
                            },
                            scope.clone(),
                        ));
                    }
                }
            }
            for (variant, (rule, _)) in rules.iter().enumerate() {
                let lexeme = id("f", index, variant, class);
                builder.lexeme(lexeme.as_str(), class, &candidate.lemma, &candidate.stem);
                builder.accent(lexeme.as_str(), rule);
            }
            case_rules.insert(index, rules);
        }
        builder.install()?;
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            let (mut chosen, _) = unfit_groups[&index].clone();
            let mut complete = true;
            let mut seen_scopes: BTreeSet<String> = BTreeSet::new();
            for (variant, (rule, scope)) in case_rules[&index].iter().enumerate() {
                if !seen_scopes.insert(rule.scope.clone()) {
                    continue;
                }
                let group = scoped_cells(&candidate.cells, scope);
                if group.is_empty() {
                    continue;
                }
                let found = case_rules[&index]
                    .iter()
                    .enumerate()
                    .skip(variant)
                    .filter(|(_, (candidate_rule, _))| candidate_rule.scope == rule.scope)
                    .find(|(variant, _)| {
                        let lexeme = id("f", index, *variant, class);
                        group.iter().all(|cell| {
                            !cell_reproduced(liturgical, &lexeme, cell, false).is_empty()
                        })
                    });
                match found {
                    Some((_, (rule, _))) => chosen.push(rule.clone()),
                    None => {
                        complete = false;
                        break;
                    }
                }
            }
            if complete && !chosen.is_empty() {
                combined.insert(index, chosen);
            }
        }
    }

    // Phase D.
    let indices: Vec<usize> = combined.keys().copied().collect();
    for chunk in indices.chunks(2000) {
        let mut builder = ArtifactBuilder::load(root)?;
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            let lexeme = id("d", index, 0, class);
            builder.lexeme(lexeme.as_str(), class, &candidate.lemma, &candidate.stem);
            for rule in &combined[&index] {
                builder.accent(lexeme.as_str(), rule);
            }
        }
        builder.install()?;
        for &index in chunk {
            let candidate = &mut candidates[index];
            let class = &classes[candidate.class_index];
            let lexeme = id("d", index, 0, class);
            if candidate
                .cells
                .iter()
                .all(|cell| !cell_reproduced(liturgical, &lexeme, cell, false).is_empty())
            {
                candidate.status = ClusterStatus::Fit;
                candidate.accent = combined[&index].clone();
            } else {
                candidate.note = "per-group rules do not combine".into();
            }
        }
    }
    // Phase E: collateral. An admitted lexeme is analysed for every surface
    // it generates, so a gap surface it generates only inexactly would move
    // from unregistered-lemma to an engine class — a regression under
    // --check. Every fit candidate's whole paradigm is checked against the
    // gap surfaces it would reach.
    let fit: Vec<usize> = window
        .iter()
        .copied()
        .filter(|index| candidates[*index].status == ClusterStatus::Fit)
        .collect();
    for chunk in fit.chunks(2000) {
        let mut builder = ArtifactBuilder::load(root)?;
        for &index in chunk {
            let candidate = &candidates[index];
            let class = &classes[candidate.class_index];
            let lexeme = id("e", index, 0, class);
            builder.lexeme(lexeme.as_str(), class, &candidate.lemma, &candidate.stem);
            for rule in &candidate.accent {
                builder.accent(lexeme.as_str(), rule);
            }
        }
        builder.install()?;
        for &index in chunk {
            let candidate = &mut candidates[index];
            let class = &classes[candidate.class_index];
            let lexeme = id("e", index, 0, class);
            let claimed: BTreeSet<&str> = candidate
                .cells
                .iter()
                .map(|cell| cell.key.as_str())
                .collect();
            let mut collateral: Vec<String> = Vec::new();
            for cell in class_cells(class) {
                let Ok(forms) = liturgical.form_by_id(&lexeme, cell) else {
                    continue;
                };
                for variant in forms.variants() {
                    let key: String = loose_key(&variant.printed).into_iter().collect();
                    let Some(surfaces) = gap_surfaces.get(&key) else {
                        continue;
                    };
                    for surface in surfaces {
                        if claimed.contains(surface.as_str()) {
                            continue;
                        }
                        let reproduced = forms
                            .variants()
                            .iter()
                            .any(|variant| surfaces_match(surface, &variant.printed));
                        if !reproduced && !collateral.contains(surface) {
                            collateral.push(surface.clone());
                        }
                    }
                }
            }
            if !collateral.is_empty() {
                candidate.status = ClusterStatus::Collateral;
                collateral.sort();
                candidate.note = format!(
                    "would analyse without reproducing: {}",
                    collateral.join(" ")
                );
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Hypotheses TSV
// ---------------------------------------------------------------------------

fn render_hypotheses(proposal: &Proposal) -> String {
    let mut output = String::from(
        "# synodal-gold-hypotheses.tsv — (lemma, class) hypotheses for the gold gap, clustered by\n\
         # the attested cells each would clear and ranked by cells per admission. Generated by\n\
         # cargo xtask synodal-gold propose; consumed by cargo xtask synodal-gold admit. Ranking\n\
         # orders work only; the replay under docs/SYNODAL_GOLD_ORACLE.md is the reviewer.\n",
    );
    let _ = writeln!(
        output,
        "# segmentation hypotheses below the --min-cells floor (not verified this run): {}",
        proposal.below_floor
    );
    output.push_str(HYPOTHESES_HEADER);
    output.push('\n');
    for (index, cluster) in proposal.clusters.iter().enumerate() {
        let accent: Vec<String> = cluster
            .accent
            .iter()
            .map(|rule| format!("{}|{}|{}", rule.scope, rule.placement, rule.mark))
            .collect();
        let _ = writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            index + 1,
            cluster.id,
            cluster.status.code(),
            cluster.class.pos,
            cluster.class.class,
            cluster.class.gender,
            cluster.class.aspect,
            cluster.lemma,
            cluster.stem,
            cluster.cells.len(),
            cluster.token_keys.join(";"),
            cluster.paradigm_keys.join(";"),
            accent.join(";"),
            cluster.evidence.join(";"),
            cluster.note
        );
    }
    output
}

/// Reads a hypotheses TSV back for `admit`. Attested cells are rebuilt from
/// the keys: the admit replay re-derives pass/fail from the oracles, so the
/// file carries identity, class, and accent rules only.
fn parse_hypotheses(content: &str) -> Result<Vec<Cluster>, Box<dyn Error>> {
    let classes = productive_classes();
    let mut clusters = Vec::new();
    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() || line == HYPOTHESES_HEADER {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 15 {
            return Err(format!("short hypotheses row: {line}").into());
        }
        let class = classes
            .iter()
            .find(|class| {
                class.pos == fields[3]
                    && class.class == fields[4]
                    && class.gender == fields[5]
                    && class.aspect == fields[6]
            })
            .ok_or_else(|| format!("unknown class in hypotheses row: {line}"))?
            .clone();
        let split = |value: &str| -> Vec<String> {
            value
                .split(';')
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        };
        let accent = split(fields[12])
            .iter()
            .map(|rule| {
                let parts: Vec<&str> = rule.split('|').collect();
                if parts.len() != 3 {
                    return Err(format!("malformed accent rule {rule:?}"));
                }
                Ok(AccentRuleSpec {
                    scope: parts[0].to_owned(),
                    placement: parts[1].to_owned(),
                    mark: parts[2].to_owned(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        clusters.push(Cluster {
            id: fields[1].to_owned(),
            status: ClusterStatus::parse(fields[2])
                .ok_or_else(|| format!("unknown status in hypotheses row: {line}"))?,
            class,
            lemma: fields[7].to_owned(),
            stem: fields[8].to_owned(),
            cells: Vec::new(),
            token_keys: split(fields[10]),
            paradigm_keys: split(fields[11]),
            accent,
            evidence: split(fields[13]),
            note: fields[14].to_owned(),
        });
    }
    Ok(clusters)
}

fn print_propose_summary(proposal: &Proposal, output: &str, seconds: f64) {
    let clusters = &proposal.clusters;
    let mut by_status: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for cluster in clusters {
        let entry = by_status.entry(cluster.status.code()).or_default();
        entry.0 += 1;
        entry.1 += cluster.cells.len();
    }
    let fit: Vec<&Cluster> = clusters
        .iter()
        .filter(|cluster| cluster.status == ClusterStatus::Fit)
        .collect();
    let fit_cells: usize = fit.iter().map(|cluster| cluster.cells.len()).sum();
    println!(
        "synodal-gold propose: {} clusters over {} attested cells; {} fit clusters clearing {} cells; wrote {output}",
        clusters.len(),
        clusters
            .iter()
            .map(|cluster| cluster.cells.len())
            .sum::<usize>(),
        fit.len(),
        fit_cells
    );
    for (status, (count, cells)) in &by_status {
        println!("  {status}\t{count} clusters\t{cells} cells");
    }
    let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for cluster in &fit {
        *histogram.entry(cluster.cells.len()).or_default() += 1;
    }
    let distribution: Vec<String> = histogram
        .iter()
        .map(|(cells, count)| format!("{cells}:{count}"))
        .collect();
    println!(
        "  fit cells-per-cluster distribution: {}",
        distribution.join(" ")
    );
    println!(
        "  segmentation hypotheses below the min-cells floor: {}",
        proposal.below_floor
    );
    println!("  propose runtime: {seconds:.1}s");
}

// ---------------------------------------------------------------------------
// Admission
// ---------------------------------------------------------------------------

struct Admission {
    cluster: Cluster,
    lexeme_id: String,
    evidence_id: String,
    candidate_id: String,
    source_id: String,
    citation: String,
}

pub(crate) struct AdmitOutcome {
    kept: Vec<String>,
    rejected: Vec<(String, Vec<String>)>,
    cleared_by_class: BTreeMap<(String, String), usize>,
    residue_rows: usize,
    replay_seconds: f64,
}

/// Passage/section → candidate id for the two gold sources, so every
/// admission's evidence row links to a real candidate record.
fn candidate_index(root: &Path) -> Result<BTreeMap<(String, String), String>, Box<dyn Error>> {
    let mut index = BTreeMap::new();
    for (source, relative) in [
        (PONOMAR_SOURCE, PONOMAR_RELATIVE),
        (ALYPY_SOURCE, ALYPY_RELATIVE),
    ] {
        let path = root.join(relative);
        let content = fs::read_to_string(&path).map_err(|error| {
            format!(
                "read {}: {error} (admit needs the gold source candidates)",
                path.display()
            )
        })?;
        for line in content.lines() {
            let Some(candidate) = extract_json_string(line, "candidate_id") else {
                continue;
            };
            let Some(passage) = extract_json_string(line, "passage") else {
                continue;
            };
            index
                .entry((source.to_owned(), passage.to_owned()))
                .or_insert_with(|| candidate.to_owned());
        }
    }
    Ok(index)
}

fn extract_json_string<'a>(line: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("\"{field}\":\"");
    let start = line.find(&marker)? + marker.len();
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

fn admit_batch(root: &Path, clusters: &[Cluster]) -> Result<AdmitOutcome, Box<dyn Error>> {
    let candidates = candidate_index(root)?;
    let mut admissions: Vec<Admission> = Vec::new();
    let mut rejected: Vec<(String, Vec<String>)> = Vec::new();
    for cluster in clusters {
        let (source_id, citation) = if cluster.paradigm_keys.is_empty() {
            (
                PONOMAR_SOURCE,
                cluster.evidence.first().cloned().unwrap_or_default(),
            )
        } else {
            (
                ALYPY_SOURCE,
                cluster.evidence.first().cloned().unwrap_or_default(),
            )
        };
        let Some(candidate_id) = candidates
            .get(&(source_id.to_owned(), citation.clone()))
            .cloned()
        else {
            rejected.push((
                cluster.id.clone(),
                vec![format!("no {source_id} candidate record for {citation:?}")],
            ));
            continue;
        };
        admissions.push(Admission {
            lexeme_id: format!("synodal:{}:{}", cluster.class.pos, cluster.id),
            evidence_id: cluster.id.clone(),
            candidate_id,
            source_id: source_id.to_owned(),
            citation,
            cluster: cluster.clone(),
        });
    }
    if admissions.is_empty() {
        return Ok(AdmitOutcome {
            kept: Vec::new(),
            rejected,
            cleared_by_class: BTreeMap::new(),
            residue_rows: 0,
            replay_seconds: 0.0,
        });
    }
    let committed = committed_gap(root)?;
    write_admissions(root, &admissions)?;
    regenerate_and_install(root)?;
    // The scope is everything an admission can touch: its cluster's cells
    // plus every oracle row the new lexeme analyses or resolves (a lexeme
    // reaches surfaces beyond its cluster, and a reached row that fails is a
    // regression the full gate would reject).
    let mut scope = Scope::from_keys(admissions.iter().flat_map(|admission| {
        admission
            .cluster
            .token_keys
            .iter()
            .chain(admission.cluster.paradigm_keys.iter())
            .cloned()
    }));
    scope.lemmas = admissions
        .iter()
        .map(|admission| admission.lexeme_id.clone())
        .collect();
    let first = synodal_gold::replay(root, &scope)?;
    let mut replay_seconds = first.seconds;
    let attribution = attribute_rows(root, &first, &admissions)?;
    let mut kept: Vec<String> = Vec::new();
    let mut revert: Vec<&Admission> = Vec::new();
    for admission in &admissions {
        // A kept admission clears its cluster and leaves every other row it
        // reaches in its committed class (or better).
        let mut failures = Vec::new();
        for row in &first.gap {
            let key = (row.oracle.to_owned(), row.key.clone());
            let in_cluster = match row.oracle {
                "token" => admission.cluster.token_keys.contains(&row.key),
                _ => admission.cluster.paradigm_keys.contains(&row.key),
            };
            let reached = attribution
                .get(&key)
                .is_some_and(|ids| ids.contains(&admission.lexeme_id));
            if !in_cluster && !reached {
                continue;
            }
            let regressed = committed
                .get(&key)
                .is_none_or(|reason| reason != row.reason);
            if in_cluster || regressed {
                failures.push(format!(
                    "{}\t{}\t{}\t{}\t{}",
                    row.oracle, row.key, row.reason, row.engine_output, row.expected
                ));
            }
        }
        if failures.is_empty() {
            kept.push(admission.lexeme_id.clone());
        } else {
            rejected.push((admission.cluster.id.clone(), failures));
            revert.push(admission);
        }
    }
    if !revert.is_empty() {
        remove_admissions(root, &revert)?;
        regenerate_and_install(root)?;
    }
    // The authoritative scoped replay of what landed.
    scope.lemmas = kept.clone();
    let after = synodal_gold::replay(root, &scope)?;
    replay_seconds += after.seconds;
    let regressed: Vec<String> = after
        .gap
        .iter()
        .filter(|row| {
            committed
                .get(&(row.oracle.to_owned(), row.key.clone()))
                .is_none_or(|reason| reason != row.reason)
        })
        .map(|row| row.render())
        .collect();
    if !regressed.is_empty() {
        return Err(format!(
            "admit left {} covered rows failing with a class absent from the committed gap; first: {}",
            regressed.len(),
            regressed[0]
        )
        .into());
    }
    let mut cleared_by_class: BTreeMap<(String, String), usize> = BTreeMap::new();
    let still_failing: BTreeSet<(String, String)> = after
        .gap
        .iter()
        .map(|row| (row.oracle.to_owned(), row.key.clone()))
        .collect();
    for (oracle, key) in &after.covered {
        if still_failing.contains(&(oracle.clone(), key.clone())) {
            continue;
        }
        if let Some(reason) = committed.get(&(oracle.clone(), key.clone())) {
            *cleared_by_class
                .entry((oracle.clone(), reason.clone()))
                .or_default() += 1;
        }
    }
    write_rejected(root, &rejected)?;
    Ok(AdmitOutcome {
        kept,
        rejected,
        cleared_by_class,
        residue_rows: 0,
        replay_seconds,
    })
}

/// Which admitted lexemes each failing row reaches: a token row through the
/// analyser (the freshly installed registry), a paradigm row through its
/// headword resolution.
fn attribute_rows(
    root: &Path,
    replay: &synodal_gold::Replay,
    admissions: &[Admission],
) -> Result<Attribution, Box<dyn Error>> {
    let admitted: BTreeSet<&str> = admissions
        .iter()
        .map(|admission| admission.lexeme_id.as_str())
        .collect();
    let analyzer =
        synodal_church_slavonic_dictionary::coverage::Analyzer::new(Inflector::default())
            .map_err(|error| format!("build analyzer: {error}"))?;
    let paradigm_rows = synodal_gold::load_paradigm_oracle(root)?;
    let headwords: BTreeMap<&str, &str> = paradigm_rows
        .iter()
        .map(|row| (row.key.as_str(), row.headword.as_str()))
        .collect();
    let mut attribution: Attribution = BTreeMap::new();
    for row in &replay.gap {
        let ids: BTreeSet<String> = match row.oracle {
            "token" => analyzer
                .analyze_profile(
                    &synodal_gold::nfc(&row.key),
                    OrthographyProfile::SynodalLiturgical,
                )
                .unwrap_or_default()
                .iter()
                .map(|analysis| analysis.lexeme.id().as_str().to_owned())
                .filter(|id| admitted.contains(id.as_str()))
                .collect(),
            _ => headwords
                .get(row.key.as_str())
                .and_then(|headword| paradigm_lemma(headword))
                .and_then(|lemma| {
                    synodal_church_slavonic::lookup(&lemma)
                        .or_else(|_| synodal_church_slavonic::lookup(&strip_accents(&lemma)))
                        .ok()
                })
                .map(|lexeme| lexeme.id().as_str().to_owned())
                .filter(|id| admitted.contains(id.as_str()))
                .into_iter()
                .collect(),
        };
        if !ids.is_empty() {
            attribution.insert((row.oracle.to_owned(), row.key.clone()), ids);
        }
    }
    Ok(attribution)
}

fn print_admit_summary(outcome: &AdmitOutcome, seconds: f64) {
    println!(
        "synodal-gold admit: kept {} admissions ({} rules, {} residue rows), rejected {}; replay {:.1}s, total {seconds:.1}s",
        outcome.kept.len(),
        outcome.kept.len(),
        outcome.residue_rows,
        outcome.rejected.len(),
        outcome.replay_seconds
    );
    for ((oracle, class), count) in &outcome.cleared_by_class {
        println!("  cleared\t{oracle}\t{class}\t{count}");
    }
    for (id, failures) in outcome.rejected.iter().take(10) {
        println!("  rejected {id}: {}", failures[0]);
    }
    if outcome.rejected.len() > 10 {
        println!(
            "  ... {} more in {REJECTED_RELATIVE}",
            outcome.rejected.len() - 10
        );
    }
}

/// Appends the admission rows: one lexeme, one reviewed-evidence row, and
/// the accent paradigm rows (one block per lexeme, uniform evidence).
fn write_admissions(root: &Path, admissions: &[Admission]) -> Result<(), Box<dyn Error>> {
    let data = root.join("data/synodal");
    let mut lexemes = String::new();
    let mut evidence = String::new();
    let mut accents = String::new();
    for admission in admissions {
        let cluster = &admission.cluster;
        let _ = writeln!(
            lexemes,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\tsynodal-russian",
            admission.lexeme_id,
            cluster.lemma,
            cluster.class.pos,
            cluster.class.class,
            cluster.stem,
            cluster.class.gender,
            cluster.class.aspect,
            admission.source_id
        );
        let _ = writeln!(
            evidence,
            "{}\t{}\t{}\t{}\treviewed\tsynodal-russian\tsynodal-gold admit: class {} reproduces every attested gold cell of this lexeme ({} cells; {}); accents from the printed cells.",
            admission.evidence_id,
            admission.candidate_id,
            admission.source_id,
            admission.citation,
            cluster.class.class,
            cluster.token_keys.len() + cluster.paradigm_keys.len(),
            cluster.evidence.join(", ")
        );
        for rule in &cluster.accent {
            let _ = writeln!(
                accents,
                "{}\tsynodal-accent:{}\t{}\t{}\t{}\t\t{}\t{}\t{}\tsynodal-russian\tsynodal-russian",
                admission.lexeme_id,
                cluster.id,
                rule.scope,
                rule.placement,
                rule.mark,
                admission.evidence_id,
                admission.source_id,
                admission.citation
            );
        }
    }
    append(&data.join("lexemes.tsv"), &lexemes)?;
    append(&data.join("reviewed_evidence.tsv"), &evidence)?;
    append(&data.join("accent_paradigms.tsv"), &accents)?;
    Ok(())
}

fn append(path: &Path, rows: &str) -> Result<(), Box<dyn Error>> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut content = fs::read_to_string(path)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(rows);
    fs::write(path, content)?;
    Ok(())
}

/// Removes every row an admission wrote (matched by its lexeme or evidence id
/// in the first column).
fn remove_admissions(root: &Path, admissions: &[&Admission]) -> Result<(), Box<dyn Error>> {
    let data = root.join("data/synodal");
    let lexeme_ids: BTreeSet<&str> = admissions
        .iter()
        .map(|admission| admission.lexeme_id.as_str())
        .collect();
    let evidence_ids: BTreeSet<&str> = admissions
        .iter()
        .map(|admission| admission.evidence_id.as_str())
        .collect();
    for (file, ids) in [
        ("lexemes.tsv", &lexeme_ids),
        ("accent_paradigms.tsv", &lexeme_ids),
        ("reviewed_evidence.tsv", &evidence_ids),
    ] {
        let path = data.join(file);
        let content = fs::read_to_string(&path)?;
        let kept: Vec<&str> = content
            .lines()
            .filter(|line| !ids.contains(line.split('\t').next().unwrap_or_default()))
            .collect();
        fs::write(&path, format!("{}\n", kept.join("\n")))?;
    }
    Ok(())
}

/// Regenerates both registries from the curated data and installs the
/// morphology artifact in-process, so the next replay reads the new data
/// without recompiling anything. The dictionary registry is regenerated for
/// consistency (admissions do not touch it).
fn regenerate_and_install(root: &Path) -> Result<(), Box<dyn Error>> {
    let data = root.join("data/synodal");
    church_slavonic_extractor::synodal::generate_registry(&data, &root.join(ARTIFACT_RELATIVE))?;
    church_slavonic_extractor::synodal::generate_dictionary_registry(
        &data,
        &root.join(DICTIONARY_RELATIVE),
    )?;
    let artifact = fs::read_to_string(root.join(ARTIFACT_RELATIVE))?;
    synodal_church_slavonic::install_registry_override(artifact)?;
    crate::synodal::write_extraction_report(root)?;
    Ok(())
}

fn write_rejected(root: &Path, rejected: &[(String, Vec<String>)]) -> Result<(), Box<dyn Error>> {
    let mut output = String::from(
        "# synodal-gold-rejected-hypotheses.tsv — hypotheses synodal-gold admit reverted because\n\
         # their class did not reproduce every attested cell of their cluster. One row per failing\n\
         # cell. Regenerated by every admit run.\n\
         cluster_id\toracle\tkey\treason\tengine_output\texpected\n",
    );
    for (id, failures) in rejected {
        for failure in failures {
            let _ = writeln!(output, "{id}\t{failure}");
        }
    }
    fs::write(root.join(REJECTED_RELATIVE), output)?;
    Ok(())
}
