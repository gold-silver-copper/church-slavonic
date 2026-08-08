#![forbid(unsafe_code)]

use old_church_slavonic::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, ClosedClassCell, FiniteTense,
    FiniteVerbCell, Gender, ImperativeCell, LParticipleCell, NounCell, NounClass, Number,
    NumberRestriction, PartOfSpeech, ParticipleKind, Person, VerbClass,
};
use old_church_slavonic_core::adjective::AdjectiveLexeme;
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_extractor::extract::{
    check_registry, load_registry, refresh, registry_with_overrides,
};
use old_church_slavonic_extractor::schema::{FormRow, LexemeRow, Registry};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("refresh-data") => {
            let dump = required_path_flag(&mut args, "--dump")?;
            refresh(&dump, &workspace_root()?)
        }
        Some("check-registry") => check_registry(&workspace_root()?),
        Some("extraction-report") => extraction_report(),
        Some("accuracy") => accuracy(&mut args),
        Some("accuracy-ud") => accuracy_ud(&mut args),
        Some("dump-paradigms") => dump_paradigms(args.next()),
        Some("diff-paradigms") => {
            let before = args.next().ok_or("diff-paradigms needs BEFORE")?;
            let after = args.next().ok_or("diff-paradigms needs AFTER")?;
            diff_paradigms(Path::new(&before), Path::new(&after))
        }
        Some("examples") => examples(),
        Some("speed") => speed(),
        Some("guard-witnesses") => guard_witnesses(),
        Some("check-all") => check_all(),
        Some("help") | Some("-h") | Some("--help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown xtask command: {other}").into()),
    }
}

fn required_path_flag(
    args: &mut impl Iterator<Item = String>,
    expected: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let flag = args.next().ok_or(format!("expected {expected} PATH"))?;
    if flag != expected {
        return Err(format!("expected {expected}, found {flag}").into());
    }
    Ok(PathBuf::from(
        args.next()
            .ok_or(format!("expected a path after {expected}"))?,
    ))
}

fn extraction_report() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let markdown = fs::read_to_string(root.join("reports/extraction-coverage.md"))?;
    print!("{markdown}");
    Ok(())
}

#[derive(Debug, Serialize)]
struct AccuracyReport {
    schema_version: u32,
    dictionary: DictionaryAccuracy,
    oov: OovAccuracy,
    extraction_exclusions: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct DictionaryAccuracy {
    lexemes: usize,
    cells: usize,
    variants: usize,
    reachable_variants: usize,
    exact_variant_order_cells: usize,
    primary_correct_cells: usize,
    ambiguous_bare_lemma_pos_pairs: usize,
    cells_by_source: BTreeMap<String, usize>,
    paradigm_cell_sets_correct: usize,
}

#[derive(Debug, Default, Serialize)]
struct OovAccuracy {
    development: BTreeMap<String, Slice>,
    test: BTreeMap<String, Slice>,
    development_by_cell: BTreeMap<String, Slice>,
    test_by_cell: BTreeMap<String, Slice>,
    skipped_cells: usize,
}

#[derive(Debug, Default, Clone, Serialize)]
struct Slice {
    correct: usize,
    normalized_correct: usize,
    total: usize,
}

fn accuracy(args: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let registry_path = accuracy_registry_path(args, &root)?;
    let report = evaluate_accuracy(&root, &registry_path)?;
    let json = serde_json::to_vec_pretty(&report)?;
    let markdown = accuracy_markdown(&report);
    fs::write(root.join("reports/accuracy.json"), json)?;
    fs::write(root.join("reports/accuracy.md"), markdown.as_bytes())?;
    print!("{markdown}");
    Ok(())
}

fn evaluate_accuracy(root: &Path, registry_path: &Path) -> Result<AccuracyReport, Box<dyn Error>> {
    let mut registry = load_registry(registry_path)?;
    if registry_path == root.join("data/extracted") {
        registry = registry_with_overrides(registry, &root.join("data/overrides.tsv"))?;
    }
    let dictionary = dictionary_accuracy(&registry)?;
    ensure_dictionary_integrity(&dictionary)?;
    let oov = oov_accuracy(&registry);
    let extraction: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("reports/extraction-coverage.json"))?)?;
    let extraction_exclusions = serde_json::from_value(
        extraction
            .get("dropped_by_reason")
            .cloned()
            .ok_or("extraction report has no dropped_by_reason")?,
    )?;
    Ok(AccuracyReport {
        schema_version: 2,
        dictionary,
        oov,
        extraction_exclusions,
    })
}

fn ensure_dictionary_integrity(dictionary: &DictionaryAccuracy) -> Result<(), Box<dyn Error>> {
    if dictionary.reachable_variants != dictionary.variants {
        return Err("not every accepted dictionary variant reaches the public facade".into());
    }
    if dictionary.exact_variant_order_cells != dictionary.cells {
        return Err("source variant order changed in the public facade".into());
    }
    if dictionary.primary_correct_cells != dictionary.cells {
        return Err("source-order primary variants changed in the public facade".into());
    }
    if dictionary.paradigm_cell_sets_correct != dictionary.lexemes {
        return Err("dictionary paradigms and public cell getters disagree".into());
    }
    Ok(())
}

fn accuracy_registry_path(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let Some(flag) = args.next() else {
        return Ok(root.join("data/extracted"));
    };
    if flag != "--dump" {
        return Err(format!("accuracy expected --dump PATH, found {flag}").into());
    }
    let path = PathBuf::from(args.next().ok_or("accuracy --dump needs a path")?);
    if args.next().is_some() {
        return Err("accuracy received unexpected extra arguments".into());
    }
    if path.is_dir() {
        return Ok(path);
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("data/extracted/source.json"))?)?;
    let expected_bytes = metadata["bytes"].as_u64();
    if Some(fs::metadata(&path)?.len()) != expected_bytes {
        return Err(
            "raw dump does not match the committed source byte length; refresh first".into(),
        );
    }
    let expected_sha = metadata["sha256"]
        .as_str()
        .ok_or("committed source metadata has no sha256")?;
    let mut source = File::open(&path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if format!("{:x}", hasher.finalize()) != expected_sha {
        return Err("raw dump SHA-256 does not match the committed source; refresh first".into());
    }
    Ok(root.join("data/extracted"))
}

fn dictionary_accuracy(registry: &Registry) -> Result<DictionaryAccuracy, Box<dyn Error>> {
    let grouped = grouped_forms(registry);
    let mut reachable = 0;
    let mut ordered = 0;
    let mut primary = 0;
    let mut cells_by_source = BTreeMap::new();
    for ((id, feature), expected) in &grouped {
        let actual = public_cell_by_id(id, feature)?;
        let expected_values = expected
            .iter()
            .map(|row| (row.form.as_str(), row.romanization.as_str()))
            .collect::<Vec<_>>();
        let actual_values = actual
            .variants
            .iter()
            .map(|variant| {
                (
                    variant.text.as_str(),
                    variant.romanization.as_deref().unwrap_or(""),
                )
            })
            .collect::<Vec<_>>();
        reachable += expected_values
            .iter()
            .filter(|expected| actual_values.contains(expected))
            .count();
        ordered += usize::from(expected_values == actual_values);
        primary += usize::from(expected_values.first() == actual_values.first());
        let source = match actual.source {
            old_church_slavonic::FormSource::DictionaryTable => "dictionary-table",
            old_church_slavonic::FormSource::ManualOverride => "manual-override",
            old_church_slavonic::FormSource::DictionaryMetadataRule { .. } => {
                "dictionary-metadata-rule"
            }
            old_church_slavonic::FormSource::ExplicitMetadataRule { .. } => {
                "explicit-metadata-rule"
            }
            old_church_slavonic::FormSource::OovPrediction { .. } => "oov-prediction",
        };
        *cells_by_source.entry(source.to_string()).or_insert(0) += 1;
    }
    let mut alias_pos: BTreeMap<(&str, &str), BTreeSet<&str>> = BTreeMap::new();
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|row| (row.id.as_str(), row.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    for alias in &registry.aliases {
        if let Some(pos) = pos_by_id.get(alias.lexeme_id.as_str()) {
            alias_pos
                .entry((alias.key.as_str(), *pos))
                .or_default()
                .insert(alias.lexeme_id.as_str());
        }
    }
    let mut paradigm_cell_sets_correct = 0;
    for lexeme in &registry.lexemes {
        let paradigm = old_church_slavonic::dictionary_paradigm_by_id(&lexeme.id)?;
        let start = (lexeme.id.clone(), String::new());
        let end = (lexeme.id.clone(), "\u{10ffff}".to_string());
        let expected = grouped
            .range(start..=end)
            .map(|((_id, feature), _)| feature.as_str())
            .collect::<BTreeSet<_>>();
        let actual = paradigm
            .cells
            .iter()
            .map(|(feature, _)| feature.as_str())
            .collect::<BTreeSet<_>>();
        paradigm_cell_sets_correct += usize::from(expected == actual);
    }
    Ok(DictionaryAccuracy {
        lexemes: registry.lexemes.len(),
        cells: grouped.len(),
        variants: registry.forms.len(),
        reachable_variants: reachable,
        exact_variant_order_cells: ordered,
        primary_correct_cells: primary,
        ambiguous_bare_lemma_pos_pairs: alias_pos.values().filter(|ids| ids.len() > 1).count(),
        cells_by_source,
        paradigm_cell_sets_correct,
    })
}

fn public_cell_by_id(
    id: &str,
    feature: &str,
) -> Result<old_church_slavonic::FormSet, Box<dyn Error>> {
    if let Some(cell) = parse_noun_cell(feature) {
        return Ok(old_church_slavonic::noun_by_id(id, cell)?);
    }
    if let Some(cell) = parse_adjective_cell(feature) {
        return Ok(old_church_slavonic::adjective_by_id(id, cell)?);
    }
    if feature == "adj:comparative:citation" {
        return Ok(old_church_slavonic::adjective_comparatives_by_id(id)?);
    }
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "finite", tense, person, number] => Ok(old_church_slavonic::finite_verb_by_id(
            id,
            FiniteVerbCell {
                tense: parse_tense(tense).ok_or("invalid finite tense")?,
                person: parse_person(person).ok_or("invalid finite person")?,
                number: parse_number(number).ok_or("invalid finite number")?,
            },
        )?),
        ["verb", "imperative", person, number] => Ok(old_church_slavonic::imperative_by_id(
            id,
            ImperativeCell {
                person: parse_person(person).ok_or("invalid imperative person")?,
                number: parse_number(number).ok_or("invalid imperative number")?,
            },
        )?),
        ["verb", "l-participle", gender, number] => Ok(old_church_slavonic::l_participle_by_id(
            id,
            LParticipleCell {
                gender: parse_gender_code(gender).ok_or("invalid l-participle gender")?,
                number: parse_number(number).ok_or("invalid l-participle number")?,
            },
        )?),
        ["verb", "participle", kind, "citation"] => {
            Ok(old_church_slavonic::participle_citation_by_id(
                id,
                parse_participle_kind(kind).ok_or("invalid participle kind")?,
            )?)
        }
        ["verb", "infinitive"] => Ok(old_church_slavonic::infinitive_by_id(id)?),
        ["verb", "supine"] => Ok(old_church_slavonic::supine_by_id(id)?),
        ["verb", "verbal-noun"] => Ok(old_church_slavonic::verbal_noun_by_id(id)?),
        ["decl", pos, case, number, rest @ ..] => {
            let part_of_speech = match *pos {
                "pron" => PartOfSpeech::Pronoun,
                "num" => PartOfSpeech::Numeral,
                "det" => PartOfSpeech::Determiner,
                _ => return Err("invalid closed-class part of speech".into()),
            };
            let mut gender = None;
            let mut person = None;
            for value in rest.iter().copied() {
                if let Some(value) = parse_gender_code(value) {
                    gender = Some(value);
                } else if let Some(value) = parse_person(value) {
                    person = Some(value);
                } else {
                    return Err(format!("invalid closed-class feature segment: {value}").into());
                }
            }
            Ok(old_church_slavonic::closed_class_by_id(
                id,
                part_of_speech,
                ClosedClassCell {
                    case: parse_case(case).ok_or("invalid closed-class case")?,
                    number: parse_number(number).ok_or("invalid closed-class number")?,
                    gender,
                    person,
                },
            )?)
        }
        _ => Err(format!("no typed public resolver for accepted feature: {feature}").into()),
    }
}

fn parse_tense(value: &str) -> Option<FiniteTense> {
    match value {
        "present" => Some(FiniteTense::Present),
        "imperfect" => Some(FiniteTense::Imperfect),
        "aorist" => Some(FiniteTense::Aorist),
        _ => None,
    }
}

fn parse_person(value: &str) -> Option<Person> {
    match value {
        "1" => Some(Person::First),
        "2" => Some(Person::Second),
        "3" => Some(Person::Third),
        _ => None,
    }
}

fn parse_gender_code(value: &str) -> Option<Gender> {
    match value {
        "m" => Some(Gender::Masculine),
        "f" => Some(Gender::Feminine),
        "n" => Some(Gender::Neuter),
        _ => None,
    }
}

fn parse_participle_kind(value: &str) -> Option<ParticipleKind> {
    match value {
        "present-active" => Some(ParticipleKind::PresentActive),
        "present-passive" => Some(ParticipleKind::PresentPassive),
        "past-active" => Some(ParticipleKind::PastActive),
        "past-passive" => Some(ParticipleKind::PastPassive),
        _ => None,
    }
}

fn oov_accuracy(registry: &Registry) -> OovAccuracy {
    let grouped = grouped_forms(registry);
    let mut out = OovAccuracy::default();
    for lexeme in &registry.lexemes {
        let test = fnv1a(lexeme.key.as_bytes()) % 5 == 0;
        let (destination, by_cell) = if test {
            (&mut out.test, &mut out.test_by_cell)
        } else {
            (&mut out.development, &mut out.development_by_cell)
        };
        match lexeme.pos.as_str() {
            "noun" => evaluate_oov_noun(
                lexeme,
                &grouped,
                destination,
                by_cell,
                &mut out.skipped_cells,
            ),
            "adj" => evaluate_oov_adjective(
                lexeme,
                &grouped,
                destination,
                by_cell,
                &mut out.skipped_cells,
            ),
            "verb" => evaluate_oov_verb(
                lexeme,
                &grouped,
                destination,
                by_cell,
                &mut out.skipped_cells,
            ),
            _ => {}
        }
    }
    out
}

fn evaluate_oov_noun(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    destination: &mut BTreeMap<String, Slice>,
    by_cell: &mut BTreeMap<String, Slice>,
    skipped: &mut usize,
) {
    let class = match row.class.as_str() {
        "o-m-hard" => NounClass::OMasculineHard,
        "o-n-hard" => NounClass::ONeuterHard,
        "a-hard" => NounClass::AHard,
        "jo-m-soft" => NounClass::JoMasculineSoft,
        "jo-n-soft" => NounClass::JoNeuterSoft,
        "ja-soft" => NounClass::JaSoft,
        "i-f" => NounClass::IFeminine,
        "i-m" => NounClass::IMasculine,
        "u-m" => NounClass::UMasculine,
        "n-m" => NounClass::NMasculine,
        "n-n" => NounClass::NNeuter,
        "nt-n" => NounClass::NtNeuter,
        "r-n" => NounClass::RStem,
        "s-n" => NounClass::SNeuter,
        "v-f" => NounClass::VFeminine,
        _ => {
            *skipped +=
                count_lexeme_features(row, grouped, |feature| parse_noun_cell(feature).is_some());
            return;
        }
    };
    let gender = match row.gender.as_str() {
        "m" => Gender::Masculine,
        "f" => Gender::Feminine,
        "n" => Gender::Neuter,
        _ => {
            *skipped +=
                count_lexeme_features(row, grouped, |feature| parse_noun_cell(feature).is_some());
            return;
        }
    };
    let animacy = match row.animacy.as_str() {
        "an" => Some(Animacy::Animate),
        "in" => Some(Animacy::Inanimate),
        _ => None,
    };
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    for ((_id, feature), expected) in grouped.range(start..=end) {
        let Some(cell) = parse_noun_cell(feature) else {
            continue;
        };
        if gender == Gender::Masculine && cell.case == Case::Accusative && animacy.is_none() {
            *skipped += 1;
            continue;
        }
        let lexeme = NounLexeme {
            lemma: row.lemma.clone(),
            class,
            gender,
            animacy: animacy.unwrap_or(Animacy::Inanimate),
            number_restriction: parse_restriction(&row.number_restriction),
        };
        let Ok(predicted) = old_church_slavonic_core::noun::decline(&lexeme, cell) else {
            *skipped += 1;
            continue;
        };
        score_prediction(
            destination,
            &row.class,
            by_cell,
            &format!("noun/{}/{feature}", row.class),
            expected,
            &predicted.text,
        );
    }
}

fn evaluate_oov_adjective(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    destination: &mut BTreeMap<String, Slice>,
    by_cell: &mut BTreeMap<String, Slice>,
    skipped: &mut usize,
) {
    let class = match row.class.as_str() {
        "adj-hard" => AdjectiveClass::Hard,
        "adj-soft" => AdjectiveClass::Soft,
        _ => {
            *skipped += count_lexeme_features(row, grouped, |feature| {
                parse_adjective_cell(feature).is_some()
            });
            return;
        }
    };
    let lexeme = AdjectiveLexeme {
        lemma: row.lemma.clone(),
        class,
    };
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    for ((_id, feature), expected) in grouped.range(start..=end) {
        let Some(cell) = parse_adjective_cell(feature) else {
            continue;
        };
        let Ok(predicted) = old_church_slavonic_core::adjective::decline(&lexeme, cell) else {
            *skipped += 1;
            continue;
        };
        let rule_slice = format!(
            "adj-{}-{}",
            if class == AdjectiveClass::Hard {
                "hard"
            } else {
                "soft"
            },
            cell.form.code()
        );
        score_prediction(
            destination,
            &rule_slice,
            by_cell,
            &format!("adj/{rule_slice}/{feature}"),
            expected,
            &predicted.text,
        );
    }
}

fn evaluate_oov_verb(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    destination: &mut BTreeMap<String, Slice>,
    by_cell: &mut BTreeMap<String, Slice>,
    skipped: &mut usize,
) {
    let class = parse_productive_verb_class(&row.class);
    let present_stem = class.and_then(|class| {
        grouped
            .get(&(row.id.clone(), "verb:finite:present:2:sg".to_string()))
            .and_then(|forms| {
                forms
                    .iter()
                    .find_map(|form| derive_present_stem(class, &form.form))
            })
    });
    let aorist_stem = grouped
        .get(&(row.id.clone(), "verb:l-participle:m:sg".to_string()))
        .and_then(|forms| {
            forms
                .iter()
                .find_map(|form| derive_l_participle_stem(&form.form))
        });
    let lexeme = VerbLexeme {
        lemma: row.lemma.clone(),
        class: class.unwrap_or(VerbClass::Irregular),
        present_stem,
        aorist_stem,
    };
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    for ((_id, feature), expected) in grouped.range(start..=end) {
        if let Some(cell) = parse_finite_verb_cell(feature) {
            if cell.tense != FiniteTense::Present {
                continue;
            }
            if cell.person == Person::Second && cell.number == Number::Singular {
                continue;
            }
            let Some(class) = class else {
                *skipped += 1;
                continue;
            };
            let Ok(predicted) = old_church_slavonic_core::verb::finite(&lexeme, cell) else {
                *skipped += 1;
                continue;
            };
            let rule_slice = format!("verb-{}-present", class.code());
            score_prediction(
                destination,
                &rule_slice,
                by_cell,
                &format!("verb/{rule_slice}/{feature}"),
                expected,
                &predicted.text,
            );
            continue;
        }
        if feature == "verb:infinitive" {
            match old_church_slavonic_core::verb::infinitive(&lexeme) {
                Ok(predicted) => score_prediction(
                    destination,
                    "verb-infinitive",
                    by_cell,
                    "verb/verb-infinitive/verb:infinitive",
                    expected,
                    &predicted.text,
                ),
                Err(_) => *skipped += 1,
            }
            continue;
        }
        if feature == "verb:supine" {
            match old_church_slavonic_core::verb::supine(&lexeme) {
                Ok(predicted) => score_prediction(
                    destination,
                    "verb-supine",
                    by_cell,
                    "verb/verb-supine/verb:supine",
                    expected,
                    &predicted.text,
                ),
                Err(_) => *skipped += 1,
            }
            continue;
        }
        let Some(cell) = parse_l_participle_cell(feature) else {
            continue;
        };
        if cell.gender == Gender::Masculine && cell.number == Number::Singular {
            continue;
        }
        match old_church_slavonic_core::verb::l_participle(&lexeme, cell) {
            Ok(predicted) => score_prediction(
                destination,
                "verb-l-participle",
                by_cell,
                &format!("verb/verb-l-participle/{feature}"),
                expected,
                &predicted.text,
            ),
            Err(_) => *skipped += 1,
        }
    }
}

fn parse_productive_verb_class(value: &str) -> Option<VerbClass> {
    match value {
        "IA1" => Some(VerbClass::IA1),
        "IA2" => Some(VerbClass::IA2),
        "II1" => Some(VerbClass::II1),
        "II2" => Some(VerbClass::II2),
        "II3" => Some(VerbClass::II3),
        _ => None,
    }
}

fn derive_present_stem(class: VerbClass, second_singular: &str) -> Option<String> {
    let ending = match class {
        VerbClass::IA1 | VerbClass::IA2 => "еши",
        VerbClass::II1 | VerbClass::II2 | VerbClass::II3 => "иши",
        _ => return None,
    };
    second_singular
        .strip_suffix(ending)
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn derive_l_participle_stem(masculine_singular: &str) -> Option<String> {
    masculine_singular
        .strip_suffix("лъ")
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn parse_finite_verb_cell(feature: &str) -> Option<FiniteVerbCell> {
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "finite", tense, person, number] => Some(FiniteVerbCell {
            tense: parse_tense(tense)?,
            person: parse_person(person)?,
            number: parse_number(number)?,
        }),
        _ => None,
    }
}

fn parse_l_participle_cell(feature: &str) -> Option<LParticipleCell> {
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "l-participle", gender, number] => Some(LParticipleCell {
            gender: parse_gender_code(gender)?,
            number: parse_number(number)?,
        }),
        _ => None,
    }
}

fn count_lexeme_features(
    row: &LexemeRow,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    predicate: impl Fn(&str) -> bool,
) -> usize {
    let start = (row.id.clone(), String::new());
    let end = (row.id.clone(), "\u{10ffff}".to_string());
    grouped
        .range(start..=end)
        .filter(|((_id, feature), _)| predicate(feature))
        .count()
}

fn score_prediction(
    destination: &mut BTreeMap<String, Slice>,
    rule_slice: &str,
    by_cell: &mut BTreeMap<String, Slice>,
    cell_slice: &str,
    expected: &[&FormRow],
    predicted: &str,
) {
    let exact = expected.iter().any(|form| form.form == predicted);
    let normalized = expected
        .iter()
        .any(|form| normalized_equal(&form.form, predicted));
    for slice in [
        destination.entry(rule_slice.to_string()).or_default(),
        by_cell.entry(cell_slice.to_string()).or_default(),
    ] {
        slice.total += 1;
        slice.correct += usize::from(exact);
        slice.normalized_correct += usize::from(normalized);
    }
}

fn normalized_equal(left: &str, right: &str) -> bool {
    old_church_slavonic::orthography::lookup_key(left).ok()
        == old_church_slavonic::orthography::lookup_key(right).ok()
}

fn grouped_forms(registry: &Registry) -> BTreeMap<(String, String), Vec<&FormRow>> {
    let mut grouped: BTreeMap<(String, String), Vec<&FormRow>> = BTreeMap::new();
    for row in &registry.forms {
        grouped
            .entry((row.lexeme_id.clone(), row.feature.clone()))
            .or_default()
            .push(row);
    }
    for rows in grouped.values_mut() {
        rows.sort_by_key(|row| row.rank);
    }
    grouped
}

fn parse_noun_cell(feature: &str) -> Option<NounCell> {
    let mut parts = feature.split(':');
    if parts.next()? != "noun" {
        return None;
    }
    let case = parse_case(parts.next()?)?;
    let number = parse_number(parts.next()?)?;
    parts.next().is_none().then_some(NounCell { case, number })
}

fn parse_adjective_cell(feature: &str) -> Option<AdjectiveCell> {
    let mut parts = feature.split(':');
    if parts.next()? != "adj" {
        return None;
    }
    let form = match parts.next()? {
        "short" => AdjectiveForm::Short,
        "long" => AdjectiveForm::Long,
        _ => return None,
    };
    let case = parse_case(parts.next()?)?;
    let number = parse_number(parts.next()?)?;
    let gender = match parts.next()? {
        "m" => Gender::Masculine,
        "f" => Gender::Feminine,
        "n" => Gender::Neuter,
        _ => return None,
    };
    let animacy = match parts.next()? {
        "an" => Animacy::Animate,
        "in" => Animacy::Inanimate,
        _ => return None,
    };
    parts.next().is_none().then_some(AdjectiveCell {
        case,
        number,
        gender,
        animacy,
        form,
    })
}

fn parse_case(value: &str) -> Option<Case> {
    match value {
        "nom" => Some(Case::Nominative),
        "gen" => Some(Case::Genitive),
        "dat" => Some(Case::Dative),
        "acc" => Some(Case::Accusative),
        "ins" => Some(Case::Instrumental),
        "loc" => Some(Case::Locative),
        "voc" => Some(Case::Vocative),
        _ => None,
    }
}

fn parse_number(value: &str) -> Option<Number> {
    match value {
        "sg" => Some(Number::Singular),
        "du" => Some(Number::Dual),
        "pl" => Some(Number::Plural),
        _ => None,
    }
}

fn parse_restriction(value: &str) -> NumberRestriction {
    match value {
        "sg" => NumberRestriction::SingularOnly,
        "du" => NumberRestriction::DualOnly,
        "pl" => NumberRestriction::PluralOnly,
        _ => NumberRestriction::All,
    }
}

fn accuracy_markdown(report: &AccuracyReport) -> String {
    let dictionary = &report.dictionary;
    let mut out = String::from("# Accuracy\n\n");
    out.push_str("Dictionary round-trip and OOV prediction are separate measurements.\n\n");
    out.push_str(
        "The OOV split is lemma-level: 64-bit FNV-1a of the shared normalized lemma key, \
modulo 5. Residue 0 is the fixed held-out final-evaluation partition; residues 1-4 \
are development. Homographs and parts of speech sharing a lemma key therefore cannot \
cross partitions. The held-out partition is deterministic, not cryptographically \
sealed, and must not be used for rule tuning.\n\n",
    );
    out.push_str("## Dictionary registry round-trip\n\n");
    out.push_str("| Metric | Value |\n|---|---:|\n");
    out.push_str(&format!("| lexemes | {} |\n", dictionary.lexemes));
    out.push_str(&format!("| cells | {} |\n", dictionary.cells));
    out.push_str(&format!("| variants | {} |\n", dictionary.variants));
    out.push_str(&format!(
        "| reachable variants | {} / {} |\n",
        dictionary.reachable_variants, dictionary.variants
    ));
    out.push_str(&format!(
        "| exact variant-order cells | {} / {} |\n",
        dictionary.exact_variant_order_cells, dictionary.cells
    ));
    out.push_str(&format!(
        "| primary-correct cells | {} / {} |\n",
        dictionary.primary_correct_cells, dictionary.cells
    ));
    out.push_str(&format!(
        "| ambiguous bare lemma/POS pairs | {} |\n",
        dictionary.ambiguous_bare_lemma_pos_pairs
    ));
    out.push_str(&format!(
        "| complete dictionary paradigm key sets | {} / {} |\n\n",
        dictionary.paradigm_cell_sets_correct, dictionary.lexemes
    ));
    out.push_str("Cells by public provenance:\n\n");
    for (source, count) in &dictionary.cells_by_source {
        out.push_str(&format!("- `{source}`: {count}\n"));
    }
    out.push('\n');
    out.push_str(
        "Verb present and l-participle slices use the source 2nd-singular present and \
masculine-singular l-participle, respectively, only as lexical stem metadata. Those two \
metadata cells are excluded from scoring. This matches the explicit-stem OOV API.\n\n",
    );
    for (title, slices, by_cell) in [
        (
            "Development OOV",
            &report.oov.development,
            &report.oov.development_by_cell,
        ),
        ("Held-out OOV", &report.oov.test, &report.oov.test_by_cell),
    ] {
        out.push_str(&format!(
            "## {title}\n\n| Rule slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |\n|---|---:|---:|---:|---:|---:|\n"
        ));
        let mut macro_exact = 0.0;
        let mut macro_normalized = 0.0;
        for (class, slice) in slices {
            let rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.correct as f64 / slice.total as f64
            };
            let normalized_rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.normalized_correct as f64 / slice.total as f64
            };
            macro_exact += rate;
            macro_normalized += normalized_rate;
            out.push_str(&format!(
                "| `{class}` | {} | {} | {} | {rate:.2}% | {normalized_rate:.2}% |\n",
                slice.correct, slice.normalized_correct, slice.total
            ));
        }
        let classes = slices.len().max(1) as f64;
        out.push_str(&format!(
            "\nMacro average across reported rule slices: {:.2}% exact, {:.2}% normalized.\n",
            macro_exact / classes,
            macro_normalized / classes
        ));
        out.push_str(
            "\n### POS, class, and cell detail\n\n| Cell slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |\n|---|---:|---:|---:|---:|---:|\n",
        );
        for (cell, slice) in by_cell {
            let rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.correct as f64 / slice.total as f64
            };
            let normalized_rate = if slice.total == 0 {
                0.0
            } else {
                100.0 * slice.normalized_correct as f64 / slice.total as f64
            };
            out.push_str(&format!(
                "| `{cell}` | {} | {} | {} | {rate:.2}% | {normalized_rate:.2}% |\n",
                slice.correct, slice.normalized_correct, slice.total
            ));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "Skipped OOV cells requiring unavailable lexical metadata: {}.\n",
        report.oov.skipped_cells
    ));
    out.push_str("\n## Extraction exclusions\n\n");
    for (reason, count) in &report.extraction_exclusions {
        out.push_str(&format!("- `{reason}`: {count}\n"));
    }
    out
}

fn accuracy_ud(args: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let path = required_path_flag(args, "--path")?;
    if !path.exists() {
        return Err(format!("UD path does not exist: {}", path.display()).into());
    }
    let files = conllu_files(&path)?;
    let mut tokens = 0usize;
    let mut compatible = 0usize;
    let mut exact = 0usize;
    let mut normalized = 0usize;
    for file in &files {
        for line in fs::read_to_string(file)?.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let columns: Vec<_> = line.split('\t').collect();
            if columns.len() != 10 || columns[0].contains(['-', '.']) {
                continue;
            }
            tokens += 1;
            let (surface, lemma, upos, features) = (columns[1], columns[2], columns[3], columns[5]);
            let pos = match upos {
                "NOUN" | "PROPN" => PartOfSpeech::Noun,
                "ADJ" => PartOfSpeech::Adjective,
                "VERB" | "AUX" => PartOfSpeech::Verb,
                "PRON" => PartOfSpeech::Pronoun,
                "NUM" => PartOfSpeech::Numeral,
                "DET" => PartOfSpeech::Determiner,
                _ => continue,
            };
            let Some(feature) = ud_feature_key(pos, features) else {
                continue;
            };
            let candidates = old_church_slavonic::lookup(lemma, pos)?;
            if candidates.is_empty() {
                continue;
            }
            compatible += 1;
            let generated = candidates
                .iter()
                .filter_map(|candidate| {
                    old_church_slavonic::dictionary_form_by_id(&candidate.id, &feature).ok()
                })
                .flat_map(|forms| forms.variants)
                .collect::<Vec<_>>();
            exact += usize::from(generated.iter().any(|form| form.text == surface));
            normalized += usize::from(
                generated
                    .iter()
                    .any(|form| normalized_equal(&form.text, surface)),
            );
        }
    }
    println!(
        "UD diagnostic (CC BY-NC-SA input, not bundled): {exact}/{compatible} raw exact; {normalized}/{compatible} NFC/lowercase exact; {tokens} tokens scanned across {} files",
        files.len()
    );
    Ok(())
}

fn ud_feature_key(pos: PartOfSpeech, features: &str) -> Option<String> {
    let map = features
        .split('|')
        .filter_map(|feature| feature.split_once('='))
        .collect::<BTreeMap<_, _>>();
    if pos == PartOfSpeech::Verb {
        return ud_verb_feature_key(&map);
    }
    let case = match *map.get("Case")? {
        "Nom" => "nom",
        "Gen" => "gen",
        "Dat" => "dat",
        "Acc" => "acc",
        "Ins" => "ins",
        "Loc" => "loc",
        "Voc" => "voc",
        _ => return None,
    };
    let number = match *map.get("Number")? {
        "Sing" => "sg",
        "Dual" => "du",
        "Plur" => "pl",
        _ => return None,
    };
    match pos {
        PartOfSpeech::Noun => Some(format!("noun:{case}:{number}")),
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner => {
            let gender = map.get("Gender").and_then(|value| ud_gender(value));
            let person = map.get("Person").and_then(|value| parse_person(value));
            Some(
                ClosedClassCell {
                    case: parse_case(case)?,
                    number: parse_number(number)?,
                    gender: gender.and_then(parse_gender_code),
                    person,
                }
                .key(pos),
            )
        }
        // PROIEL does not encode the OCS short/long adjective dimension, so an
        // adjective bundle is not compatible with either public feature key.
        PartOfSpeech::Adjective | PartOfSpeech::Verb => None,
    }
}

fn ud_verb_feature_key(map: &BTreeMap<&str, &str>) -> Option<String> {
    match *map.get("VerbForm")? {
        "Inf" => Some("verb:infinitive".to_string()),
        "Sup" => Some("verb:supine".to_string()),
        "PartRes" => {
            let gender = ud_gender(map.get("Gender")?)?;
            let number = match *map.get("Number")? {
                "Sing" => "sg",
                "Dual" => "du",
                "Plur" => "pl",
                _ => return None,
            };
            Some(format!("verb:l-participle:{gender}:{number}"))
        }
        "Fin" => {
            let person = *map.get("Person")?;
            if !matches!(person, "1" | "2" | "3") {
                return None;
            }
            let number = match *map.get("Number")? {
                "Sing" => "sg",
                "Dual" => "du",
                "Plur" => "pl",
                _ => return None,
            };
            match *map.get("Mood")? {
                "Imp" => Some(format!("verb:imperative:{person}:{number}")),
                "Ind" if map.get("Tense").copied() == Some("Pres") => {
                    Some(format!("verb:finite:present:{person}:{number}"))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn ud_gender(value: &str) -> Option<&'static str> {
    match value {
        "Masc" => Some("m"),
        "Fem" => Some("f"),
        "Neut" => Some("n"),
        _ => None,
    }
}

fn conllu_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                stack.push(entry?.path());
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "conllu")
        {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

fn dump_paradigms(name: Option<String>) -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let registry = load_registry(&root.join("data/extracted"))?;
    let directory = root.join("target/paradigm-fingerprint");
    fs::create_dir_all(&directory)?;
    let path = directory.join(format!(
        "{}.tsv",
        name.unwrap_or_else(|| "dump".to_string())
    ));
    let mut output = String::from("lexeme_id\tpos\tfeature\trank\tform\tromanization\n");
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|row| (row.id.as_str(), row.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    for row in &registry.forms {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row.lexeme_id,
            pos_by_id
                .get(row.lexeme_id.as_str())
                .copied()
                .unwrap_or("?"),
            row.feature,
            row.rank,
            row.form,
            row.romanization
        ));
    }
    fs::write(&path, output.as_bytes())?;
    println!(
        "wrote {} ({} form variants)",
        path.display(),
        registry.forms.len()
    );
    Ok(())
}

fn diff_paradigms(before: &Path, after: &Path) -> Result<(), Box<dyn Error>> {
    let load = |path: &Path| -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        Ok(fs::read_to_string(path)?
            .lines()
            .skip(1)
            .filter_map(|line| {
                let columns = line.split('\t').collect::<Vec<_>>();
                (columns.len() == 6).then(|| {
                    (
                        format!(
                            "{}\t{}\t{}\t{}",
                            columns[0], columns[1], columns[2], columns[3]
                        ),
                        format!("{}\t{}", columns[4], columns[5]),
                    )
                })
            })
            .collect())
    };
    let before_rows = load(before)?;
    let after_rows = load(after)?;
    let mut changes = 0usize;
    for (key, old) in &before_rows {
        match after_rows.get(key) {
            Some(new) if new != old => {
                println!("changed\t{key}\t{old}\t->\t{new}");
                changes += 1;
            }
            None => {
                println!("removed\t{key}\t{old}");
                changes += 1;
            }
            _ => {}
        }
    }
    for (key, new) in &after_rows {
        if !before_rows.contains_key(key) {
            println!("added\t{key}\t{new}");
            changes += 1;
        }
    }
    eprintln!("{changes} changed variants");
    Ok(())
}

fn examples() -> Result<(), Box<dyn Error>> {
    run_cargo(&["run", "-p", "old-church-slavonic", "--example", "basic"])?;
    run_cargo(&["run", "-p", "old-church-slavonic", "--example", "tour"])
}

fn speed() -> Result<(), Box<dyn Error>> {
    run_cargo(&[
        "run",
        "-p",
        "old-church-slavonic",
        "--example",
        "speedmark",
        "--release",
    ])
}

fn check_all() -> Result<(), Box<dyn Error>> {
    run_cargo(&["fmt", "--all", "--", "--check"])?;
    run_cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])?;
    run_cargo(&["test", "--workspace", "--all-features"])?;
    run_cargo(&["test", "--workspace", "--doc"])?;
    let root = workspace_root()?;
    check_registry(&root)?;
    check_accuracy_report(&root)?;
    check_runtime_boundaries(&root)?;
    check_attribution(&root)?;
    examples()
}

fn check_accuracy_report(root: &Path) -> Result<(), Box<dyn Error>> {
    let report = evaluate_accuracy(root, &root.join("data/extracted"))?;
    let expected_json = serde_json::to_vec_pretty(&report)?;
    let expected_markdown = accuracy_markdown(&report);
    if fs::read(root.join("reports/accuracy.json"))? != expected_json
        || fs::read_to_string(root.join("reports/accuracy.md"))? != expected_markdown
    {
        return Err("committed accuracy reports are stale; run cargo xtask accuracy".into());
    }
    println!("accuracy reports: current");
    Ok(())
}

fn check_runtime_boundaries(root: &Path) -> Result<(), Box<dyn Error>> {
    for relative in [
        "crates/old-church-slavonic-core/src",
        "crates/old-church-slavonic/src",
    ] {
        let mut stack = vec![root.join(relative)];
        while let Some(path) = stack.pop() {
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    stack.push(entry?.path());
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path)?;
                for forbidden in [
                    "std::fs",
                    "std::net",
                    "TcpStream",
                    "UdpSocket",
                    "reqwest",
                    "ureq",
                ] {
                    if source.contains(forbidden) {
                        return Err(format!(
                            "runtime I/O/network boundary violation in {}: {forbidden}",
                            path.display()
                        )
                        .into());
                    }
                }
            }
        }
    }
    println!("runtime boundary: no I/O or network APIs");
    Ok(())
}

fn check_attribution(root: &Path) -> Result<(), Box<dyn Error>> {
    let package = root.join("crates/old-church-slavonic");
    let attribution = fs::read_to_string(package.join("ATTRIBUTION.md"))?;
    let source: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("data/extracted/source.json"))?)?;
    let sha = source["sha256"]
        .as_str()
        .ok_or("source metadata has no SHA-256")?;
    if !attribution.contains(sha)
        || !attribution.contains("English Wiktionary")
        || !attribution.contains("CC BY-SA 4.0")
        || !attribution.contains("creativecommons.org/licenses/by-sa/4.0/legalcode")
        || !attribution.contains("source was modified")
    {
        return Err("published attribution is missing source identity or license".into());
    }
    let manifest = fs::read_to_string(package.join("Cargo.toml"))?;
    if !manifest.contains("CC-BY-SA-4.0") {
        return Err("published manifest omits the bundled data license".into());
    }
    for required in [
        "ATTRIBUTION.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "generated/**",
    ] {
        if !manifest.contains(required) {
            return Err(format!("published manifest omits required artifact: {required}").into());
        }
    }
    if !fs::read_to_string(package.join("LICENSE-MIT"))?.contains("MIT License")
        || !fs::read_to_string(package.join("LICENSE-APACHE"))?.contains("Apache License")
    {
        return Err("published code license texts are incomplete".into());
    }
    println!("package attribution: current");
    Ok(())
}

fn guard_witnesses() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let witness_root = std::env::temp_dir().join(format!(
        "old-church-slavonic-guard-witnesses-{}",
        std::process::id()
    ));
    if witness_root.exists() {
        fs::remove_dir_all(&witness_root)?;
    }
    let result = (|| -> Result<(), Box<dyn Error>> {
        copy_guard_fixture(&root, &witness_root)?;

        let generated = "crates/old-church-slavonic/generated/registry.rs";
        let mut changed = fs::read_to_string(witness_root.join(generated))?;
        changed.push_str("\n// stale generated witness\n");
        fs::write(witness_root.join(generated), changed)?;
        require_guard_failure(
            "generated registry freshness",
            check_registry(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, generated)?;

        let forms = "data/extracted/forms.tsv";
        let mut changed = fs::read_to_string(witness_root.join(forms))?;
        let duplicate = changed
            .lines()
            .nth(1)
            .ok_or("forms fixture has no data row")?
            .to_string();
        changed.push_str(&duplicate);
        changed.push('\n');
        fs::write(witness_root.join(forms), changed)?;
        require_guard_failure("duplicate cell/rank", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        rewrite_form_row(
            &witness_root.join(forms),
            |columns| columns.get(3).is_some_and(|form| !form.is_empty()),
            |columns| columns[3] = "—".to_string(),
        )?;
        require_guard_failure("sentinel public form", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        rewrite_form_row(
            &witness_root.join(forms),
            |columns| columns.get(3).is_some_and(|form| !form.is_empty()),
            |columns| columns[3] = "сло{{{2}}}во".to_string(),
        )?;
        require_guard_failure("MediaWiki markup form", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        rewrite_form_row(
            &witness_root.join(forms),
            |columns| columns[0].starts_with("обѣдъ|noun|") && columns[1] == "noun:nom:sg",
            |columns| columns[3] = "несъвпадение".to_string(),
        )?;
        require_guard_failure("canonical noun citation", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        swap_first_variant_pair(&witness_root.join(forms))?;
        require_guard_failure("source variant order", check_accuracy_report(&witness_root))?;
        restore_guard_file(&root, &witness_root, forms)?;

        require_guard_failure(
            "coverage floor",
            old_church_slavonic_extractor::validate::coverage(
                old_church_slavonic_extractor::validate::MIN_ACCEPTED_LEXEMES - 1,
                old_church_slavonic_extractor::validate::MIN_ACCEPTED_FORMS,
            ),
        )?;

        let runtime = "crates/old-church-slavonic-core/src/lib.rs";
        let mut changed = fs::read_to_string(witness_root.join(runtime))?;
        changed.push_str("\nuse std::fs;\n");
        fs::write(witness_root.join(runtime), changed)?;
        require_guard_failure(
            "runtime I/O boundary",
            check_runtime_boundaries(&witness_root),
        )?;
        restore_guard_file(&root, &witness_root, runtime)?;

        let attribution = "crates/old-church-slavonic/ATTRIBUTION.md";
        let mut changed = fs::read_to_string(witness_root.join(attribution))?;
        changed = changed.replace(
            "5bd61e747aa7aeb677af92b4e32c65476e5c6ee74bff146269460c962be5456c",
            "missing-source-hash",
        );
        fs::write(witness_root.join(attribution), changed)?;
        require_guard_failure("published attribution", check_attribution(&witness_root))?;
        restore_guard_file(&root, &witness_root, attribution)?;

        let extraction_report = "reports/extraction-coverage.json";
        let mut changed: serde_json::Value =
            serde_json::from_slice(&fs::read(witness_root.join(extraction_report))?)?;
        changed["accepted_forms"] = serde_json::Value::from(1_u64);
        fs::write(
            witness_root.join(extraction_report),
            serde_json::to_vec_pretty(&changed)?,
        )?;
        require_guard_failure("extraction report freshness", check_registry(&witness_root))?;
        restore_guard_file(&root, &witness_root, extraction_report)?;

        let accuracy_report = "reports/accuracy.md";
        let mut changed = fs::read_to_string(witness_root.join(accuracy_report))?;
        changed.push_str("\nstale accuracy witness\n");
        fs::write(witness_root.join(accuracy_report), changed)?;
        require_guard_failure(
            "accuracy report freshness",
            check_accuracy_report(&witness_root),
        )?;

        let registry = load_registry(&root.join("data/extracted"))?;
        let mut integrity = dictionary_accuracy(&registry)?;
        integrity.paradigm_cell_sets_correct = integrity
            .paradigm_cell_sets_correct
            .checked_sub(1)
            .ok_or("dictionary fixture has no lexemes")?;
        require_guard_failure(
            "paradigm/cell agreement",
            ensure_dictionary_integrity(&integrity),
        )?;

        for hostile in ["", "two words", "\0", &"x".repeat(4_097)] {
            if std::panic::catch_unwind(|| {
                let _ = old_church_slavonic::lookup(hostile, PartOfSpeech::Noun);
            })
            .is_err()
            {
                return Err(format!("hostile-input guard observed a panic for {hostile:?}").into());
            }
        }
        println!("guard witness observed: hostile input remains panic-free");
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&witness_root);
    result?;
    cleanup?;
    println!("guard-witnesses: all injected failures were detected and reverted");
    Ok(())
}

fn copy_guard_fixture(root: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    for relative in [
        "data/extracted",
        "crates/old-church-slavonic-core/src",
        "crates/old-church-slavonic/src",
        "crates/old-church-slavonic/generated",
    ] {
        copy_tree(&root.join(relative), &destination.join(relative))?;
    }
    for relative in [
        "data/overrides.tsv",
        "data/citation-exemptions.tsv",
        "data/SOURCES.toml",
        "reports/extraction-coverage.json",
        "reports/extraction-coverage.md",
        "reports/accuracy.json",
        "reports/accuracy.md",
        "crates/old-church-slavonic/ATTRIBUTION.md",
        "crates/old-church-slavonic/Cargo.toml",
        "crates/old-church-slavonic/LICENSE-MIT",
        "crates/old-church-slavonic/LICENSE-APACHE",
    ] {
        restore_guard_file(root, destination, relative)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn restore_guard_file(
    root: &Path,
    destination: &Path,
    relative: &str,
) -> Result<(), Box<dyn Error>> {
    let target = destination.join(relative);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(root.join(relative), target)?;
    Ok(())
}

fn require_guard_failure<E: std::fmt::Display>(
    name: &str,
    result: Result<(), E>,
) -> Result<(), Box<dyn Error>> {
    match result {
        Ok(()) => Err(format!("guard witness did not fail: {name}").into()),
        Err(error) => {
            println!("guard witness observed: {name}: {error}");
            Ok(())
        }
    }
}

fn rewrite_form_row(
    path: &Path,
    predicate: impl Fn(&[String]) -> bool,
    mutation: impl Fn(&mut [String]),
) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut changed = false;
    let mut output = String::new();
    for (line_index, line) in contents.lines().enumerate() {
        let mut columns = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        if line_index > 0 && !changed && predicate(&columns) {
            mutation(&mut columns);
            changed = true;
        }
        output.push_str(&columns.join("\t"));
        output.push('\n');
    }
    if !changed {
        return Err("guard witness could not find its target form row".into());
    }
    fs::write(path, output)?;
    Ok(())
}

fn swap_first_variant_pair(path: &Path) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut lines = contents
        .lines()
        .map(|line| line.split('\t').map(str::to_string).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let pair = (1..lines.len().saturating_sub(1)).find(|index| {
        lines[*index][0] == lines[*index + 1][0]
            && lines[*index][1] == lines[*index + 1][1]
            && lines[*index][2] == "0"
            && lines[*index + 1][2] == "1"
    });
    let index = pair.ok_or("guard witness found no multi-variant cell")?;
    let (left, right) = lines.split_at_mut(index + 1);
    for column in 3..=4 {
        std::mem::swap(&mut left[index][column], &mut right[0][column]);
    }
    let mut output = lines
        .into_iter()
        .map(|columns| columns.join("\t"))
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    fs::write(path, output)?;
    Ok(())
}

fn run_cargo(args: &[&str]) -> Result<(), Box<dyn Error>> {
    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .current_dir(workspace_root()?)
        .args(args)
        .status()?;
    require_success(status, args)
}

fn require_success(status: ExitStatus, args: &[&str]) -> Result<(), Box<dyn Error>> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed with {status}", args.join(" ")).into())
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn print_help() {
    eprintln!("cargo xtask <command>");
    eprintln!("  refresh-data --dump PATH");
    eprintln!("  check-registry");
    eprintln!("  extraction-report");
    eprintln!("  accuracy");
    eprintln!("  accuracy-ud --path UD_DIRECTORY");
    eprintln!("  dump-paradigms [NAME]");
    eprintln!("  diff-paradigms BEFORE AFTER");
    eprintln!("  examples");
    eprintln!("  speed");
    eprintln!("  guard-witnesses");
    eprintln!("  check-all");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_principal_parts_yield_explicit_verb_stems() {
        assert_eq!(
            derive_present_stem(VerbClass::IA1, "несеши").as_deref(),
            Some("нес")
        );
        assert_eq!(
            derive_present_stem(VerbClass::II1, "правиши").as_deref(),
            Some("прав")
        );
        assert_eq!(
            derive_l_participle_stem("правилъ").as_deref(),
            Some("прави")
        );
        assert_eq!(derive_present_stem(VerbClass::Root, "еси"), None);
        assert_eq!(derive_l_participle_stem("лъ"), None);
    }

    #[test]
    fn ud_mapper_accepts_only_fully_compatible_verb_bundles() {
        assert_eq!(
            ud_feature_key(
                PartOfSpeech::Verb,
                "Mood=Ind|Number=Dual|Person=1|Tense=Pres|VerbForm=Fin"
            ),
            Some("verb:finite:present:1:du".to_string())
        );
        assert_eq!(
            ud_feature_key(
                PartOfSpeech::Verb,
                "Mood=Imp|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin"
            ),
            Some("verb:imperative:2:sg".to_string())
        );
        assert_eq!(
            ud_feature_key(PartOfSpeech::Verb, "VerbForm=Inf"),
            Some("verb:infinitive".to_string())
        );
        assert_eq!(
            ud_feature_key(PartOfSpeech::Verb, "VerbForm=Sup"),
            Some("verb:supine".to_string())
        );
        assert_eq!(
            ud_feature_key(
                PartOfSpeech::Verb,
                "Gender=Fem|Number=Plur|Tense=Past|VerbForm=PartRes"
            ),
            Some("verb:l-participle:f:pl".to_string())
        );
        assert_eq!(
            ud_feature_key(
                PartOfSpeech::Verb,
                "Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"
            ),
            None
        );
    }

    #[test]
    fn ud_adjective_without_short_long_feature_is_not_guessed() {
        assert_eq!(
            ud_feature_key(
                PartOfSpeech::Adjective,
                "Case=Nom|Degree=Pos|Gender=Masc|Number=Sing"
            ),
            None
        );
    }
}
