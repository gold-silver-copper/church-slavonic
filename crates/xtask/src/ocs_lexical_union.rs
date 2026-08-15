//! Reproducible Old Church Slavonic lexical-union inventory.
//!
//! The committed ledger contains source claims, not copied paradigms. A source
//! claim is merged with a runtime identity only when the match is unique; every
//! other identity remains source-qualified so homonyms are never silently
//! collapsed.

use old_church_slavonic_core::{
    CardinalNumeralIdentity, IrregularVerbFamilyMember, PersonalPronounIdentity, RegularVerbFamily,
    RegularVerbSourceMember, TwofoldNounFamilyMember, UniqueNounFamilyMember,
    UniqueVerbFamilyMember, orthography::lookup_key,
};
use old_church_slavonic_extractor::{
    extract::canonical_lemma,
    schema::{Entry, LexemeRow},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const LEDGER_PATH: &str = "data/ocs/lexical_source_claims.tsv";
const REGULAR_VERB_PATH: &str = "data/ocs/polivanova_regular_verbs.tsv";
const JSON_REPORT_PATH: &str = "reports/ocs-lexical-union.json";
const MARKDOWN_REPORT_PATH: &str = "reports/ocs-lexical-union.md";
const KAIKKI_RUNTIME_SOURCE: &str = "english-wiktionary-ocs-kaikki-2026-07-06";
const KAIKKI_SOURCE: &str = "english-wiktionary-ocs-kaikki-2026-08-07";
const OSD_SOURCE: &str = "polivanova-osd-source";
const KAIKKI_RUNTIME_SHA256: &str =
    "5bd61e747aa7aeb677af92b4e32c65476e5c6ee74bff146269460c962be5456c";
const KAIKKI_SHA256: &str = "fb20336e716d8f29d0c53bb4cc32f35065ad973ef8b496654c72bf542f876a83";
const OSD_SHA256: &str = "f412042aafdf2a6650f52f7c42a8a971127bfefe9dfd155b36347b3e11f7f38d";
const OSD_XLS_SHA256: &str = "2cc1befe8d93324c0baa809b8b96714f158655f97c61a3e9a129920dcee27959";
const OSD_REVISION: &str = "osd.zip-last-modified-2020-01-10";
const LEDGER_HEADER: &str = "claim_id\tsource_id\tsource_record\tlemma\tlookup_key\tsource_pos\tengine_pos\tsource_class\tunion_identity\tclassification\tengine_route\tsupport_state\tevidence\tnotes";
const CLASSIFICATIONS: &[&str] = &[
    "productive",
    "closed-irregular",
    "defective",
    "indeclinable",
    "ambiguous",
    "disputed",
    "out-of-scope",
];
const SUPPORT_STATES: &[&str] = &[
    "implemented",
    "implementation-missing",
    "metadata-incomplete",
    "source-ambiguous",
    "not-applicable",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Claim {
    claim_id: String,
    source_id: String,
    source_record: String,
    lemma: String,
    lookup_key: String,
    source_pos: String,
    engine_pos: String,
    source_class: String,
    union_identity: String,
    classification: String,
    engine_route: String,
    support_state: String,
    evidence: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct IntermediateRow {
    source_id: String,
    source_revision: String,
    artifact_sha256: String,
    source_order: usize,
    raw_spelling: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u8,
    source_union_policy: &'static str,
    source_artifacts: Vec<SourceArtifact>,
    claims: usize,
    union_identities: usize,
    merged_runtime_identities: usize,
    by_source: BTreeMap<String, usize>,
    by_part_of_speech: BTreeMap<String, usize>,
    by_classification: BTreeMap<String, usize>,
    by_support_state: BTreeMap<String, usize>,
    implementation_gaps_by_route: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct SourceArtifact {
    source_id: &'static str,
    sha256: &'static str,
    row_policy: &'static str,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    let mut kaikki = root.join(
        "references/downloads/english-wiktionary-ocs/kaikki.org-dictionary-OldChurchSlavonic.jsonl",
    );
    let mut osd = root.join("data/intermediate/synodal/polivanova-osd-source.jsonl");
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--kaikki" => kaikki = PathBuf::from(args.next().ok_or("--kaikki needs PATH")?),
            "--osd-jsonl" => osd = PathBuf::from(args.next().ok_or("--osd-jsonl needs PATH")?),
            value => return Err(format!("unknown ocs-lexical-union argument {value:?}").into()),
        }
    }

    if check {
        let claims = load_ledger(&root.join(LEDGER_PATH))?;
        validate(root, &claims)?;
        let report = report(&claims);
        require_report_current(root, &report)?;
        require_regular_verbs_current(root, &osd)?;
        println!("OCS lexical source-union ledger: current");
        return Ok(());
    }

    let registry =
        old_church_slavonic_extractor::extract::load_registry(&root.join("data/extracted"))?;
    let runtime = RuntimeIndex::new(&registry.lexemes);
    let mut claims = read_runtime_registry(&registry.lexemes);
    claims.extend(read_kaikki(&kaikki, &runtime)?);
    claims.extend(read_osd(&osd, &runtime)?);
    claims.sort();
    validate_claims(&claims)?;
    fs::create_dir_all(root.join("data/ocs"))?;
    fs::write(root.join(LEDGER_PATH), render_ledger(&claims))?;
    let report = report(&claims);
    fs::write(
        root.join(JSON_REPORT_PATH),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(root.join(MARKDOWN_REPORT_PATH), render_markdown(&report))?;
    fs::write(
        root.join(REGULAR_VERB_PATH),
        render_regular_verbs(&read_regular_verbs(&osd)?)?,
    )?;
    println!("wrote {LEDGER_PATH} ({} source claims)", claims.len());
    Ok(())
}

pub(crate) fn check(root: &Path) -> Result<(), Box<dyn Error>> {
    let claims = load_ledger(&root.join(LEDGER_PATH))?;
    validate(root, &claims)?;
    require_report_current(root, &report(&claims))?;
    require_regular_verbs_current(
        root,
        &root.join("data/intermediate/synodal/polivanova-osd-source.jsonl"),
    )?;
    println!("OCS lexical source-union ledger: current");
    Ok(())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RegularVerbRow {
    source_row: usize,
    lemma: String,
    class: String,
    class_four_basic_stem: String,
}

fn read_regular_verbs(path: &Path) -> Result<Vec<RegularVerbRow>, Box<dyn Error>> {
    let reader = BufReader::new(File::open(path)?);
    let mut rows = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let row: IntermediateRow = serde_json::from_str(&line?)?;
        if row.source_id != OSD_SOURCE
            || row.source_revision != OSD_REVISION
            || row.artifact_sha256 != OSD_XLS_SHA256
            || row.source_order != index + 1
        {
            return Err(format!(
                "OSD intermediate row {} has unexpected source provenance or order",
                index + 1
            )
            .into());
        }
        if row.source_order == 1 {
            continue;
        }
        let columns = row.raw_spelling.split('\t').collect::<Vec<_>>();
        if columns.len() != 14 {
            return Err(
                format!("OSD row {} has {} columns", row.source_order, columns.len()).into(),
            );
        }
        if columns[12] != "v"
            || !is_regular_osd_verb_class(columns[11])
            || columns[10].contains(['(', ')'])
        {
            continue;
        }
        let class_four_basic_stem = if columns[11] == "4c" {
            let source = columns[7]
                .strip_suffix(".т.и")
                .ok_or_else(|| format!("class 4c OSD row {} lacks .т.и", row.source_order - 1))?;
            let stem = source
                .chars()
                .filter(|character| !matches!(character, '.' | '(' | ')' | ' '))
                .collect::<String>();
            if stem.is_empty() {
                return Err(format!(
                    "class 4c OSD row {} has an empty basic stem",
                    row.source_order - 1
                )
                .into());
            }
            stem
        } else {
            "-".to_string()
        };
        rows.push(RegularVerbRow {
            source_row: row.source_order - 1,
            lemma: clean_osd_lemma(columns[10]),
            class: columns[11].to_string(),
            class_four_basic_stem,
        });
    }
    if rows.len() != 2_297 {
        return Err(format!("expected 2297 regular OSD verbs, found {}", rows.len()).into());
    }
    Ok(rows)
}

fn render_regular_verbs(rows: &[RegularVerbRow]) -> Result<String, Box<dyn Error>> {
    let mut output = String::from("source_row\tlemma\tclass\tclass_four_basic_stem\n");
    for row in rows {
        if row.lemma.contains(['\t', '\n'])
            || row.class.contains(['\t', '\n'])
            || row.class_four_basic_stem.contains(['\t', '\n'])
        {
            return Err(format!("OSD row {} contains a TSV delimiter", row.source_row).into());
        }
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            row.source_row, row.lemma, row.class, row.class_four_basic_stem
        ));
    }
    Ok(output)
}

fn require_regular_verbs_current(root: &Path, source: &Path) -> Result<(), Box<dyn Error>> {
    let expected = render_regular_verbs(&read_regular_verbs(source)?)?;
    if fs::read_to_string(root.join(REGULAR_VERB_PATH))? != expected {
        return Err(
            format!("stale {REGULAR_VERB_PATH}; rerun cargo xtask ocs-lexical-union").into(),
        );
    }
    Ok(())
}

fn read_runtime_registry(rows: &[LexemeRow]) -> Vec<Claim> {
    rows.iter()
        .map(|row| {
            let mut claim = Claim {
                claim_id: format!("{KAIKKI_RUNTIME_SOURCE}:registry:{}", row.id),
                source_id: KAIKKI_RUNTIME_SOURCE.to_string(),
                source_record: format!("registry:{}", row.id),
                lemma: row.lemma.clone(),
                lookup_key: row.key.clone(),
                source_pos: row.pos.clone(),
                engine_pos: row.pos.clone(),
                source_class: nonempty(&row.class).to_string(),
                union_identity: row.id.clone(),
                classification: String::new(),
                engine_route: String::new(),
                support_state: String::new(),
                evidence: format!(
                    "Pinned 2026-07-06 Kaikki extraction; registry identity {}; source signature {}",
                    row.id, row.signature
                ),
                notes: String::new(),
            };
            classify_runtime(&mut claim, row);
            claim
        })
        .collect()
}

struct RuntimeIndex<'a> {
    by_page_pos: BTreeMap<(String, String), Vec<&'a LexemeRow>>,
    by_key_pos: BTreeMap<(String, String), Vec<&'a LexemeRow>>,
}

impl<'a> RuntimeIndex<'a> {
    fn new(rows: &'a [LexemeRow]) -> Self {
        let mut by_page_pos: BTreeMap<(String, String), Vec<&LexemeRow>> = BTreeMap::new();
        let mut by_key_pos: BTreeMap<(String, String), Vec<&LexemeRow>> = BTreeMap::new();
        for row in rows {
            by_page_pos
                .entry((row.page_word.clone(), row.pos.clone()))
                .or_default()
                .push(row);
            by_key_pos
                .entry((row.key.clone(), row.pos.clone()))
                .or_default()
                .push(row);
        }
        Self {
            by_page_pos,
            by_key_pos,
        }
    }

    fn kaikki(&self, entry: &Entry, pos: &str) -> Option<&'a LexemeRow> {
        let rows = self
            .by_page_pos
            .get(&(entry.word.clone(), pos.to_string()))?;
        if rows.len() == 1 {
            return rows.first().copied();
        }
        let lemma = canonical_lemma(entry, pos);
        let matches = rows
            .iter()
            .copied()
            .filter(|row| row.lemma == lemma)
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches[0])
    }

    fn by_key(&self, key: &str, positions: &[&str]) -> Option<&'a LexemeRow> {
        let matches = positions
            .iter()
            .flat_map(|pos| {
                self.by_key_pos
                    .get(&(key.to_string(), (*pos).to_string()))
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches[0])
    }
}

fn read_kaikki(path: &Path, runtime: &RuntimeIndex<'_>) -> Result<Vec<Claim>, Box<dyn Error>> {
    verify_file(path, KAIKKI_SHA256)?;
    let reader = BufReader::new(File::open(path)?);
    let mut claims = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let entry: Entry = serde_json::from_str(&line?)?;
        if entry.lang_code != "cu" {
            continue;
        }
        let source_record = format!("line:{}", index + 1);
        let claim_id = format!("{KAIKKI_SOURCE}:{source_record}");
        let source_pos = entry.pos.clone();
        let normalized_pos = normalize_pos(&entry.pos);
        let form_of = !entry.senses.is_empty()
            && entry
                .senses
                .iter()
                .all(|sense| sense.tags.iter().any(|tag| tag == "form-of"));
        let lemma = normalized_pos.map_or(entry.word.as_str(), |pos| canonical_lemma(&entry, pos));
        let key = safe_key(lemma);
        let runtime_row = normalized_pos.and_then(|pos| runtime.kaikki(&entry, pos));

        let mut claim = Claim {
            claim_id: claim_id.clone(),
            source_id: KAIKKI_SOURCE.to_string(),
            source_record,
            lemma: lemma.to_string(),
            lookup_key: key,
            source_pos,
            engine_pos: normalized_pos.unwrap_or("-").to_string(),
            source_class: runtime_row
                .map_or("-", |row| nonempty(&row.class))
                .to_string(),
            union_identity: runtime_row.map_or_else(|| claim_id.clone(), |row| row.id.clone()),
            classification: String::new(),
            engine_route: String::new(),
            support_state: String::new(),
            evidence: format!(
                "Kaikki OCS pinned line {}; extracted registry {}",
                index + 1,
                runtime_row.map_or("no safe runtime identity", |_| "runtime identity")
            ),
            notes: String::new(),
        };
        if form_of {
            classify(
                &mut claim,
                "out-of-scope",
                "form-of-source-record",
                "not-applicable",
                "The source explicitly marks every sense as form-of; this is evidence for another lexeme, not an independent lexeme identity.",
            );
        } else if normalized_pos.is_none() {
            if is_invariant_pos(&entry.pos) {
                classify(
                    &mut claim,
                    "indeclinable",
                    "invariant-part-of-speech",
                    "implemented",
                    "The source part of speech has no inflectional paradigm in the OCS completion contract.",
                );
            } else {
                classify(
                    &mut claim,
                    "out-of-scope",
                    "non-inflectional-or-unsupported-source-pos",
                    "not-applicable",
                    "The source part of speech is outside the inflectable OCS lexeme denominator.",
                );
            }
        } else if let Some(row) = runtime_row {
            classify_runtime(&mut claim, row);
        } else {
            claim.union_identity = claim.claim_id.clone();
            classify(
                &mut claim,
                "ambiguous",
                "source-record-needs-lexical-metadata",
                "source-ambiguous",
                "The pinned row has no safely attributable extracted paradigm; a bare spelling does not determine class, restrictions, or principal parts.",
            );
        }
        claims.push(claim);
    }
    Ok(claims)
}

fn read_osd(path: &Path, runtime: &RuntimeIndex<'_>) -> Result<Vec<Claim>, Box<dyn Error>> {
    let reader = BufReader::new(File::open(path)?);
    let mut claims = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let row: IntermediateRow = serde_json::from_str(&line?)?;
        if row.source_id != OSD_SOURCE
            || row.source_revision != OSD_REVISION
            || row.artifact_sha256 != OSD_XLS_SHA256
        {
            return Err(format!(
                "OSD intermediate row {} has unexpected source provenance",
                index + 1
            )
            .into());
        }
        if row.source_order != index + 1 {
            return Err(format!(
                "OSD intermediate row {} declares source order {}",
                index + 1,
                row.source_order
            )
            .into());
        }
        if row.source_order == 1 {
            continue;
        }
        let columns = row.raw_spelling.split('\t').collect::<Vec<_>>();
        if columns.len() != 14 {
            return Err(
                format!("OSD row {} has {} columns", row.source_order, columns.len()).into(),
            );
        }
        let source_record = format!("row:{}", row.source_order - 1);
        let claim_id = format!("{OSD_SOURCE}:{source_record}");
        let source_class = nonempty(columns[11]).to_string();
        let source_pos = nonempty(columns[12]).to_string();
        let raw_lemma = columns[10].trim();
        let lemma = clean_osd_lemma(raw_lemma);
        let key = safe_key(&lemma);
        let engine_positions = osd_engine_positions(columns[12]);
        let runtime_row = (key != "-")
            .then(|| runtime.by_key(&key, engine_positions))
            .flatten();
        let mut claim = Claim {
            claim_id,
            source_id: OSD_SOURCE.to_string(),
            source_record,
            lemma: lemma.clone(),
            lookup_key: key,
            source_pos: source_pos.clone(),
            engine_pos: osd_engine_pos(columns[12]).to_string(),
            source_class,
            union_identity: runtime_row.map_or_else(|| "-".to_string(), |row| row.id.clone()),
            classification: String::new(),
            engine_route: String::new(),
            support_state: String::new(),
            evidence: format!(
                "Polivanova OSD paradigmatic dictionary row {}; class {}; morphophonology {}",
                row.source_order - 1,
                nonempty(columns[11]),
                nonempty(columns[7])
            ),
            notes: String::new(),
        };
        if columns[12] == "s" || columns[11].is_empty() {
            claim.union_identity = claim.claim_id.clone();
            classify(
                &mut claim,
                "out-of-scope",
                "secondary-form-or-cross-reference",
                "not-applicable",
                "The row has no paradigmatic class and is a secondary form or cross-reference, not an independent inflectable lexeme.",
            );
        } else if raw_lemma.contains(['(', ')']) {
            claim.union_identity = claim.claim_id.clone();
            classify(
                &mut claim,
                "disputed",
                "source-parenthesized-reconstruction",
                "source-ambiguous",
                "The normalized headword retains a parenthesized segment; the engine must not silently choose whether that segment is present.",
            );
        } else {
            classify_osd(&mut claim, runtime_row);
        }
        if claim.union_identity == "-" {
            claim.union_identity = claim.claim_id.clone();
            claim.notes =
                "No unique same-key runtime identity; source-qualified identity retained."
                    .to_string();
        }
        claims.push(claim);
    }
    Ok(claims)
}

fn classify_runtime(claim: &mut Claim, row: &LexemeRow) {
    match row.pos.as_str() {
        "noun" if noun_class_is_runtime(&row.class) && metadata_complete_noun(row) => classify(
            claim,
            if row.class == "indeclinable" {
                "indeclinable"
            } else {
                "productive"
            },
            "dictionary-noun-metadata",
            "implemented",
            "The runtime identity has a typed noun class, gender, animacy, and number restriction.",
        ),
        "noun" if noun_class_is_runtime(&row.class) => classify(
            claim,
            "ambiguous",
            "dictionary-noun-metadata",
            "metadata-incomplete",
            "The declension class is typed, but gender or animacy required by the public noun specification is absent.",
        ),
        "adj" if matches!(row.class.as_str(), "adj-hard" | "adj-soft") => classify(
            claim,
            "productive",
            "dictionary-adjective-metadata",
            "implemented",
            "The runtime identity has a typed productive adjective class; exact tables remain higher-priority evidence.",
        ),
        "verb" if unique_or_irregular_verb(&row.lemma) => classify(
            claim,
            "closed-irregular",
            "reviewed-irregular-verb-family",
            "implemented",
            "The lemma belongs to the exhaustively reviewed unique or reusable-irregular verb family inventory.",
        ),
        "verb" if RegularVerbFamily::classify_source_lemma(&row.lemma).is_some() => classify(
            claim,
            "productive",
            "polivanova-regular-verb-specification",
            "implemented",
            "The runtime spelling resolves through a source-listed productive Polivanova class and complete typed principal-part specification.",
        ),
        "verb" if verb_class_is_runtime(&row.class) => classify(
            claim,
            "productive",
            "dictionary-verb-principal-parts",
            "metadata-incomplete",
            "A present class is typed, but the source-backed principal-part inventory is not complete for every verb subsystem.",
        ),
        "pron" | "num" | "det" => classify(
            claim,
            "closed-irregular",
            "reviewed-closed-class-inventory",
            "implemented",
            "The closed-class source union is routed through the exhaustively reviewed pronoun, determiner, or numeral identity inventory.",
        ),
        _ => classify(
            claim,
            "ambiguous",
            "source-class-not-normalized",
            "source-ambiguous",
            "The source class is absent or not safely normalized into a typed engine specification.",
        ),
    }
}

fn classify_osd(claim: &mut Claim, runtime: Option<&LexemeRow>) {
    let class = claim.source_class.as_str();
    match claim.source_pos.as_str() {
        "a" if class.starts_with("0/") || class == "1/a" => classify(
            claim,
            "closed-irregular",
            "reviewed-closed-nominal-identity",
            "implemented",
            "Polivanova assigns this member to a unique closed nominal paradigm implemented by the reviewed closed-class APIs.",
        ),
        "a" if class == "2/p" || class == "2/p+" => classify(
            claim,
            "productive",
            "polivanova-pronominal-specification",
            "implemented",
            "The complete class 2/p inventory and its deformations are routed through typed pronominal specifications.",
        ),
        "a" if class == "2/a" => classify(
            claim,
            "productive",
            "polivanova-adjective-specification",
            "implemented",
            "The class 2/a citation determines the productive hard or soft adjective specification, including both short and long paradigms.",
        ),
        "a" if class == "2/a**" => classify(
            claim,
            "productive",
            "polivanova-comparative-principal-parts",
            "metadata-incomplete",
            "The comparative class is implemented, but this source row alone does not serialize both required syncopated and expanded principal parts.",
        ),
        "n" if osd_noun_deformation_route(class).is_some() => {
            let implemented = TwofoldNounFamilyMember::classify_source_lemma(&claim.lemma)
                .is_some_and(|member| noun_deformation_class_matches(member, class));
            classify(
                claim,
                "productive",
                osd_noun_deformation_route(class).unwrap_or("unrecognized-source-class"),
                if implemented {
                    "implemented"
                } else {
                    "implementation-missing"
                },
                "Polivanova defines a productive nominal deformation represented by a dedicated typed NounClass, complete twenty-one-cell rule, and exhaustive lexical family assignment.",
            );
        }
        "n" if class.starts_with("0/") => {
            let route = unique_nominal_owner(&claim.lemma, class);
            classify(
                claim,
                "closed-irregular",
                route.unwrap_or("polivanova-unique-noun"),
                if route.is_some() {
                    "implemented"
                } else {
                    "implementation-missing"
                },
                "Polivanova's class-0 nominal is routed through its exhaustive fixed-gender unique-noun family or its already complete numeral/personal-pronoun lexical owner.",
            );
        }
        "n" if matches!(class, "2/m" | "2/n" | "2/f" | "1/m" | "1/f") => {
            if runtime.is_some_and(metadata_complete_noun) {
                classify(
                    claim,
                    "productive",
                    "dictionary-noun-metadata",
                    "implemented",
                    "The source-native class is crosswalked to a unique runtime noun identity with complete typed metadata.",
                );
            } else {
                classify(
                    claim,
                    "productive",
                    "polivanova-noun-specification",
                    "metadata-incomplete",
                    "The source supplies declension type and morphological gender, but the engine still needs a reviewed animacy and number-restriction crosswalk.",
                );
            }
        }
        "v" if class == "0" => {
            let implemented =
                UniqueVerbFamilyMember::classify_source_union_lemma(&claim.lemma).is_some();
            classify(
                claim,
                "closed-irregular",
                "polivanova-unique-verb-family",
                if implemented {
                    "implemented"
                } else {
                    "implementation-missing"
                },
                "Polivanova's class 0 is an exhaustively listed unique-verb family; every member must resolve through the reviewed family inventory.",
            );
        }
        "v" if is_irregular_osd_verb_class(class) => {
            let implemented =
                IrregularVerbFamilyMember::classify_source_lemma(&claim.lemma).is_some();
            classify(
                claim,
                if class.contains('∇') {
                    "defective"
                } else {
                    "closed-irregular"
                },
                "polivanova-listed-irregular-verb",
                if implemented {
                    "implemented"
                } else {
                    "implementation-missing"
                },
                "All 310 marked OSD rows resolve through the exhaustive §421 family inventory; the bounded §§464 and 509 anomalies have exact reviewed profiles.",
            );
        }
        "v" if is_regular_osd_verb_class(class) => {
            let member = claim
                .source_record
                .strip_prefix("row:")
                .and_then(|row| row.parse::<u16>().ok())
                .and_then(RegularVerbSourceMember::from_source_row);
            let implemented = member.is_some_and(|member| {
                member.canonical_lemma() == claim.lemma && member.class().code() == class
            });
            classify(
                claim,
                "productive",
                "polivanova-regular-verb-specification",
                if implemented {
                    "implemented"
                } else {
                    "implementation-missing"
                },
                "The OSD row has a row-addressed productive class specification; class 4c retains its otherwise unrecoverable morphophonological consonant stem.",
            );
        }
        _ => classify(
            claim,
            "ambiguous",
            "unrecognized-source-class",
            "source-ambiguous",
            "The source row does not match a reviewed OCS paradigmatic class contract.",
        ),
    }
}

fn classify(claim: &mut Claim, class: &str, route: &str, support: &str, note: &str) {
    claim.classification = class.to_string();
    claim.engine_route = route.to_string();
    claim.support_state = support.to_string();
    claim.notes = note.to_string();
}

fn normalize_pos(value: &str) -> Option<&'static str> {
    match value {
        "noun" | "name" => Some("noun"),
        "adj" => Some("adj"),
        "verb" => Some("verb"),
        "pron" => Some("pron"),
        "num" => Some("num"),
        "det" => Some("det"),
        _ => None,
    }
}

fn is_invariant_pos(value: &str) -> bool {
    matches!(
        value,
        "adv" | "prep" | "conj" | "particle" | "interj" | "postp"
    )
}

fn osd_engine_pos(value: &str) -> &'static str {
    match value {
        "a" => "adjectival",
        "n" => "noun",
        "v" => "verb",
        _ => "-",
    }
}

fn osd_engine_positions(value: &str) -> &'static [&'static str] {
    match value {
        "a" => &["adj", "pron", "num", "det"],
        "n" => &["noun"],
        "v" => &["verb"],
        _ => &[],
    }
}

fn clean_osd_lemma(value: &str) -> String {
    let value = value.split("//").next().unwrap_or(value).trim();
    let mut chars = value.chars().collect::<Vec<_>>();
    if chars.last().is_some_and(char::is_ascii_digit)
        && chars
            .get(chars.len().saturating_sub(2))
            .is_some_and(|character| !character.is_ascii_digit())
    {
        chars.pop();
    }
    chars.into_iter().collect()
}

fn safe_key(value: &str) -> String {
    if !matches!(
        old_church_slavonic_core::orthography::detect_script(value),
        old_church_slavonic_core::Script::Cyrillic | old_church_slavonic_core::Script::Glagolitic
    ) {
        return "-".to_string();
    }
    lookup_key(value).unwrap_or_else(|_| "-".to_string())
}

fn nonempty(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

fn unique_or_irregular_verb(lemma: &str) -> bool {
    UniqueVerbFamilyMember::classify_source_union_lemma(lemma).is_some()
        || IrregularVerbFamilyMember::classify_source_lemma(lemma).is_some()
}

fn noun_class_is_runtime(value: &str) -> bool {
    matches!(
        value,
        "o-m-hard"
            | "o-n-hard"
            | "jo-m-soft"
            | "jo-n-soft"
            | "a-hard"
            | "ja-soft"
            | "i-f"
            | "i-m"
            | "u-m"
            | "n-m"
            | "n-n"
            | "nt-n"
            | "r-n"
            | "s-n"
            | "v-f"
            | "indeclinable"
    )
}

fn metadata_complete_noun(row: &LexemeRow) -> bool {
    noun_class_is_runtime(&row.class)
        && matches!(row.gender.as_str(), "m" | "f" | "n")
        && matches!(row.animacy.as_str(), "an" | "in")
}

fn verb_class_is_runtime(value: &str) -> bool {
    matches!(value, "IA1" | "IA2" | "II1" | "II2" | "II3")
}

fn is_regular_osd_verb_class(value: &str) -> bool {
    matches!(value, "1" | "2" | "3" | "4c" | "4v" | "5" | "6" | "7")
}

fn osd_noun_deformation_route(value: &str) -> Option<&'static str> {
    match value {
        "2/m*" => Some("polivanova-agent-noun-deformation"),
        "2/m++" => Some("polivanova-in-noun-deformation"),
        "2/f*" => Some("polivanova-feminine-i-deformation"),
        _ => None,
    }
}

fn noun_deformation_class_matches(member: TwofoldNounFamilyMember, osd_class: &str) -> bool {
    member.source_class() == osd_class || (member.source_class() == "2/m**" && osd_class == "2/m++")
}

fn unique_nominal_owner(lemma: &str, osd_class: &str) -> Option<&'static str> {
    if UniqueNounFamilyMember::classify_source_lemma(lemma)
        .is_some_and(|member| member.source_class() == osd_class)
    {
        return Some("polivanova-unique-noun-family");
    }
    if osd_class == "0/m" && lemma == CardinalNumeralIdentity::Ten.canonical_lemma() {
        return Some("reviewed-cardinal-ten");
    }
    if osd_class == "0/s" {
        let identity = match lemma {
            "азъ" => Some(PersonalPronounIdentity::First),
            // The OSD uses the natural yeri spelling where the engine's
            // normalized grammar identity uses the digraph.
            "ты" => Some(PersonalPronounIdentity::Second),
            "сѧ" => Some(PersonalPronounIdentity::Reflexive),
            _ => None,
        };
        if identity.is_some() {
            return Some("reviewed-personal-reflexive-pronoun");
        }
    }
    None
}

fn is_irregular_osd_verb_class(value: &str) -> bool {
    value.contains(['*', '°', '↪', '↩', '#', '•', '∇']) || value.starts_with("4h")
}

fn verify_file(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err(format!(
            "{} has sha256 {actual}, expected {expected}",
            path.display()
        )
        .into());
    }
    Ok(())
}

fn render_ledger(claims: &[Claim]) -> String {
    let mut output = String::from(LEDGER_HEADER);
    output.push('\n');
    for claim in claims {
        output.push_str(
            &[
                claim.claim_id.as_str(),
                claim.source_id.as_str(),
                claim.source_record.as_str(),
                claim.lemma.as_str(),
                claim.lookup_key.as_str(),
                claim.source_pos.as_str(),
                claim.engine_pos.as_str(),
                claim.source_class.as_str(),
                claim.union_identity.as_str(),
                claim.classification.as_str(),
                claim.engine_route.as_str(),
                claim.support_state.as_str(),
                claim.evidence.as_str(),
                claim.notes.as_str(),
            ]
            .join("\t"),
        );
        output.push('\n');
    }
    output
}

fn load_ledger(path: &Path) -> Result<Vec<Claim>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(LEDGER_HEADER) {
        return Err(format!("{} has an invalid header", path.display()).into());
    }
    lines
        .enumerate()
        .map(|(index, line)| {
            let columns = line.split('\t').collect::<Vec<_>>();
            if columns.len() != 14 {
                return Err(format!(
                    "{} row {} has {} columns",
                    path.display(),
                    index + 2,
                    columns.len()
                )
                .into());
            }
            Ok(Claim {
                claim_id: columns[0].to_string(),
                source_id: columns[1].to_string(),
                source_record: columns[2].to_string(),
                lemma: columns[3].to_string(),
                lookup_key: columns[4].to_string(),
                source_pos: columns[5].to_string(),
                engine_pos: columns[6].to_string(),
                source_class: columns[7].to_string(),
                union_identity: columns[8].to_string(),
                classification: columns[9].to_string(),
                engine_route: columns[10].to_string(),
                support_state: columns[11].to_string(),
                evidence: columns[12].to_string(),
                notes: columns[13].to_string(),
            })
        })
        .collect()
}

fn validate(root: &Path, claims: &[Claim]) -> Result<(), Box<dyn Error>> {
    validate_claims(claims)?;
    let runtime_source: serde_json::Value = serde_json::from_str(&fs::read_to_string(
        root.join("data/extracted/source.json"),
    )?)?;
    if runtime_source
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        != Some(KAIKKI_RUNTIME_SHA256)
    {
        return Err("data/extracted/source.json does not match the runtime Kaikki revision".into());
    }
    let registry =
        old_church_slavonic_extractor::extract::load_registry(&root.join("data/extracted"))?;
    let runtime_ids = registry
        .lexemes
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let mapped_ids = claims
        .iter()
        .filter(|claim| claim.source_id == KAIKKI_RUNTIME_SOURCE)
        .map(|claim| claim.union_identity.as_str())
        .filter(|identity| runtime_ids.contains(identity))
        .collect::<BTreeSet<_>>();
    if mapped_ids != runtime_ids {
        return Err("OCS lexical union does not map every extracted runtime identity".into());
    }
    let by_source = count_by(claims.iter().map(|claim| claim.source_id.as_str()));
    if by_source.get(KAIKKI_RUNTIME_SOURCE) != Some(&3_081)
        || by_source.get(KAIKKI_SOURCE) != Some(&4_626)
        || by_source.get(OSD_SOURCE) != Some(&6_407)
    {
        return Err(format!("OCS lexical source counts changed: {by_source:?}").into());
    }
    let source_lock = fs::read_to_string(root.join("references/SOURCE_LOCK.tsv"))?;
    for (source, hash) in [(KAIKKI_SOURCE, KAIKKI_SHA256), (OSD_SOURCE, OSD_SHA256)] {
        if !source_lock
            .lines()
            .any(|line| line.starts_with(&format!("{source}\t")) && line.contains(hash))
        {
            return Err(format!("source lock does not pin {source} at {hash}").into());
        }
    }
    let inventory = fs::read_to_string(root.join("data/SOURCES.toml"))?;
    if !inventory.contains(KAIKKI_RUNTIME_SHA256) {
        return Err("data/SOURCES.toml does not pin the runtime Kaikki extraction".into());
    }
    Ok(())
}

fn validate_claims(claims: &[Claim]) -> Result<(), Box<dyn Error>> {
    if claims.is_empty() {
        return Err("OCS lexical source-union ledger is empty".into());
    }
    let mut ids = BTreeSet::new();
    for claim in claims {
        let fields = [
            claim.claim_id.as_str(),
            claim.source_id.as_str(),
            claim.source_record.as_str(),
            claim.lemma.as_str(),
            claim.lookup_key.as_str(),
            claim.source_pos.as_str(),
            claim.engine_pos.as_str(),
            claim.source_class.as_str(),
            claim.union_identity.as_str(),
            claim.classification.as_str(),
            claim.engine_route.as_str(),
            claim.support_state.as_str(),
            claim.evidence.as_str(),
            claim.notes.as_str(),
        ];
        if fields
            .iter()
            .any(|field| field.is_empty() || field.contains(['\t', '\n', '\r']))
        {
            return Err(format!("claim {:?} has an empty or unsafe field", claim.claim_id).into());
        }
        if !ids.insert(claim.claim_id.as_str()) {
            return Err(format!("duplicate source claim {:?}", claim.claim_id).into());
        }
        if claim.union_identity == "-" {
            return Err(format!("claim {:?} has no stable union identity", claim.claim_id).into());
        }
        if !CLASSIFICATIONS.contains(&claim.classification.as_str()) {
            return Err(format!("claim {:?} has invalid classification", claim.claim_id).into());
        }
        if !SUPPORT_STATES.contains(&claim.support_state.as_str()) {
            return Err(format!("claim {:?} has invalid support state", claim.claim_id).into());
        }
        if claim.support_state == "implemented"
            && matches!(
                claim.classification.as_str(),
                "ambiguous" | "disputed" | "out-of-scope"
            )
        {
            return Err(format!(
                "claim {:?} is implausibly marked implemented",
                claim.claim_id
            )
            .into());
        }
        if claim.source_id == OSD_SOURCE
            && claim.classification == "productive"
            && claim.source_class == "-"
        {
            return Err(format!("productive OSD claim {:?} has no class", claim.claim_id).into());
        }
    }
    Ok(())
}

fn report(claims: &[Claim]) -> Report {
    let runtime_identities = claims
        .iter()
        .filter(|claim| {
            !claim.union_identity.starts_with(KAIKKI_RUNTIME_SOURCE)
                && !claim.union_identity.starts_with(KAIKKI_SOURCE)
                && !claim.union_identity.starts_with(OSD_SOURCE)
        })
        .map(|claim| claim.union_identity.as_str())
        .collect::<BTreeSet<_>>();
    Report {
        schema_version: 1,
        source_union_policy: "Every retained inflectable identity from the older Kaikki extraction, every row in the separately pinned refreshed Kaikki artifact, and every Polivanova OSD row is represented: the older extraction through all 3,081 committed runtime identities retained from its 4,615 input rows, the newer revision through all 4,626 source rows, and the OSD through all 6,407 dictionary rows. Claims merge only through a unique normalized runtime identity; GORAZD and LOVe remain attributed crosschecks because no redistributable row-level snapshot is licensed.",
        source_artifacts: vec![
            SourceArtifact {
                source_id: KAIKKI_RUNTIME_SOURCE,
                sha256: KAIKKI_RUNTIME_SHA256,
                row_policy: "all 3,081 committed runtime identities derived from the 4,615-row pinned extraction; the original raw artifact is not duplicated in the download cache",
            },
            SourceArtifact {
                source_id: KAIKKI_SOURCE,
                sha256: KAIKKI_SHA256,
                row_policy: "all 4,626 pinned OCS rows, including explicit form-of and non-inflectional exclusions",
            },
            SourceArtifact {
                source_id: OSD_SOURCE,
                sha256: OSD_SHA256,
                row_policy: "all 6,407 paradigmatic-dictionary rows after the header, including secondary-form cross-references",
            },
        ],
        claims: claims.len(),
        union_identities: claims
            .iter()
            .map(|claim| claim.union_identity.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        merged_runtime_identities: runtime_identities.len(),
        by_source: count_by(claims.iter().map(|claim| claim.source_id.as_str())),
        by_part_of_speech: count_by(claims.iter().map(|claim| claim.engine_pos.as_str())),
        by_classification: count_by(claims.iter().map(|claim| claim.classification.as_str())),
        by_support_state: count_by(claims.iter().map(|claim| claim.support_state.as_str())),
        implementation_gaps_by_route: count_by(
            claims
                .iter()
                .filter(|claim| claim.support_state == "implementation-missing")
                .map(|claim| claim.engine_route.as_str()),
        ),
    }
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_default() += 1;
    }
    counts
}

fn render_markdown(report: &Report) -> String {
    let mut output = String::from("# OCS Lexical Source Union\n\n");
    output.push_str("This report is generated by `cargo xtask ocs-lexical-union`. The ledger records source claims rather than silently merged spellings.\n\n");
    output.push_str(&format!("- Source claims: {}\n", report.claims));
    output.push_str(&format!(
        "- Stable union identities: {}\n",
        report.union_identities
    ));
    output.push_str(&format!(
        "- Runtime identities reached by a unique merge: {}\n\n",
        report.merged_runtime_identities
    ));
    output.push_str("## Support states\n\n| State | Claims |\n|---|---:|\n");
    for (state, count) in &report.by_support_state {
        output.push_str(&format!("| `{state}` | {count} |\n"));
    }
    output.push_str("\n## Classifications\n\n| Classification | Claims |\n|---|---:|\n");
    for (classification, count) in &report.by_classification {
        output.push_str(&format!("| `{classification}` | {count} |\n"));
    }
    output.push_str("\n## Confirmed implementation gaps\n\n| Route | Claims |\n|---|---:|\n");
    for (route, count) in &report.implementation_gaps_by_route {
        output.push_str(&format!("| `{route}` | {count} |\n"));
    }
    output.push_str("\nLOVe's official 970-row export and GORAZD remain source-frontier crosschecks, not row-level denominator members: neither reviewed service states a license permitting a bundled database snapshot. This boundary does not weaken their rule-level or manual lexical evidence.\n");
    output
}

fn require_report_current(root: &Path, report: &Report) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_vec_pretty(report)?;
    if fs::read(root.join(JSON_REPORT_PATH))? != json {
        return Err(
            format!("stale {JSON_REPORT_PATH}; rerun cargo xtask ocs-lexical-union").into(),
        );
    }
    let markdown = render_markdown(report);
    if fs::read_to_string(root.join(MARKDOWN_REPORT_PATH))? != markdown {
        return Err(
            format!("stale {MARKDOWN_REPORT_PATH}; rerun cargo xtask ocs-lexical-union").into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regular_profile_form(
        lexeme: &old_church_slavonic_core::verb::VerbLexeme,
        feature: &str,
    ) -> Option<Result<String, old_church_slavonic_core::InflectionError>> {
        use old_church_slavonic_core::{
            AdjectiveCell, AdjectiveForm, Animacy, Case, FiniteTense, FiniteVerbCell, Gender,
            ImperativeCell, LParticipleCell, NounCell, Number, ParticipleCell, ParticipleKind,
            Person,
        };
        let person = |value| match value {
            "1" => Some(Person::First),
            "2" => Some(Person::Second),
            "3" => Some(Person::Third),
            _ => None,
        };
        let number = |value| match value {
            "sg" => Some(Number::Singular),
            "du" => Some(Number::Dual),
            "pl" => Some(Number::Plural),
            _ => None,
        };
        let gender = |value| match value {
            "m" => Some(Gender::Masculine),
            "f" => Some(Gender::Feminine),
            "n" => Some(Gender::Neuter),
            _ => None,
        };
        let fields = feature.split(':').collect::<Vec<_>>();
        match fields.as_slice() {
            ["verb", "finite", tense, person_code, number_code] => {
                let tense = match *tense {
                    "present" => FiniteTense::Present,
                    "imperfect" => FiniteTense::Imperfect,
                    "aorist" => FiniteTense::Aorist,
                    _ => return None,
                };
                let cell = FiniteVerbCell {
                    tense,
                    person: person(person_code)?,
                    number: number(number_code)?,
                };
                Some(old_church_slavonic_core::verb::finite(lexeme, cell).map(|form| form.text))
            }
            ["verb", "imperative", person_code, number_code] => {
                let cell = ImperativeCell {
                    person: person(person_code)?,
                    number: number(number_code)?,
                };
                Some(old_church_slavonic_core::verb::imperative(lexeme, cell).map(|form| form.text))
            }
            ["verb", "infinitive"] => {
                Some(old_church_slavonic_core::verb::infinitive(lexeme).map(|form| form.text))
            }
            ["verb", "supine"] => {
                Some(old_church_slavonic_core::verb::supine(lexeme).map(|form| form.text))
            }
            ["verb", "l-participle", gender_code, number_code] => {
                let cell = LParticipleCell {
                    gender: gender(gender_code)?,
                    number: number(number_code)?,
                };
                Some(
                    old_church_slavonic_core::verb::l_participle(lexeme, cell)
                        .map(|form| form.text),
                )
            }
            ["verb", "participle", kind, "citation"] => {
                let kind = match *kind {
                    "present-active" => ParticipleKind::PresentActive,
                    "present-passive" => ParticipleKind::PresentPassive,
                    "past-active" => ParticipleKind::PastActive,
                    "past-passive" => ParticipleKind::PastPassive,
                    _ => return None,
                };
                let cell = ParticipleCell {
                    kind,
                    adjective: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                    },
                };
                Some(old_church_slavonic_core::verb::participle(lexeme, cell).map(|form| form.text))
            }
            ["verb", "verbal-noun"] => Some(
                old_church_slavonic_core::verb::verbal_noun(
                    lexeme,
                    NounCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                    },
                )
                .map(|form| form.text),
            ),
            _ => None,
        }
    }

    #[test]
    fn osd_homonym_markers_do_not_enter_lookup_keys() {
        assert_eq!(clean_osd_lemma("притъкъ1"), "притъкъ");
        assert_eq!(clean_osd_lemma("ꙗвити//"), "ꙗвити");
        assert_eq!(clean_osd_lemma("четыре"), "четыре");
    }

    #[test]
    fn verb_class_partition_is_explicit() {
        for class in ["1", "2", "3", "4c", "4v", "5", "6", "7"] {
            assert!(is_regular_osd_verb_class(class));
            assert!(!is_irregular_osd_verb_class(class));
        }
        for class in ["3°", "3*", "4c∇", "4h", "4h*#↩"] {
            assert!(is_irregular_osd_verb_class(class));
            assert!(!is_regular_osd_verb_class(class));
        }
    }

    #[test]
    fn unique_verb_source_yeri_spellings_have_exact_family_owners() {
        for lemma in [
            "быти",
            "забыти",
            "избыти",
            "прибыти",
            "прѣбыти",
            "събыти",
            "выгънати",
        ] {
            assert!(
                UniqueVerbFamilyMember::classify_source_union_lemma(lemma).is_some(),
                "{lemma}"
            );
        }
        assert_eq!(
            UniqueVerbFamilyMember::classify_source_union_lemma("вызгънати"),
            None
        );
    }

    #[test]
    fn every_marked_osd_irregular_verb_has_an_exact_family_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let claims = load_ledger(&root.join(LEDGER_PATH)).expect("committed lexical ledger");
        let marked = claims
            .iter()
            .filter(|claim| {
                claim.source_id == OSD_SOURCE
                    && claim.source_pos == "v"
                    && is_irregular_osd_verb_class(&claim.source_class)
            })
            .collect::<Vec<_>>();

        assert_eq!(marked.len(), 310);
        for claim in marked {
            assert_eq!(
                IrregularVerbFamilyMember::classify_source_lemma(&claim.lemma)
                    .map(IrregularVerbFamilyMember::canonical_lemma),
                Some(claim.lemma.as_str()),
                "{} {}",
                claim.source_record,
                claim.lemma
            );
            assert_eq!(claim.engine_route, "polivanova-listed-irregular-verb");
            assert_eq!(claim.support_state, "implemented");
        }
    }

    #[test]
    fn regular_osd_profiles_are_crosschecked_against_matching_dictionary_cells() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let claims = load_ledger(&root.join(LEDGER_PATH)).expect("committed lexical ledger");
        let registry =
            old_church_slavonic_extractor::extract::load_registry(&root.join("data/extracted"))
                .expect("committed extracted registry");
        let identities = claims
            .iter()
            .filter(|claim| {
                claim.source_id == OSD_SOURCE
                    && claim.engine_route == "polivanova-regular-verb-specification"
            })
            .map(|claim| (claim.source_record.as_str(), claim.union_identity.as_str()))
            .collect::<BTreeMap<_, _>>();
        let mut compared = 0usize;
        let mut mismatches = Vec::new();
        for member in RegularVerbSourceMember::all() {
            let source_record = format!("row:{}", member.source_row());
            let Some(identity) = identities.get(source_record.as_str()).copied() else {
                mismatches.push(format!("{} missing ledger identity", member.source_row()));
                continue;
            };
            if identity.starts_with("polivanova-osd-source:") {
                continue;
            }
            let lexemes = member.lexemes().expect("valid source specification");
            for row in registry
                .forms
                .iter()
                .filter(|row| row.lexeme_id == identity && row.rank == 0)
            {
                let generated = lexemes
                    .iter()
                    .filter_map(|lexeme| regular_profile_form(lexeme, &row.feature))
                    .collect::<Vec<_>>();
                if generated.is_empty() {
                    continue;
                }
                compared += 1;
                if !generated.iter().any(|result| {
                    result
                        .as_ref()
                        .is_ok_and(|generated| generated == &row.form)
                }) {
                    mismatches.push(format!(
                        "row {} {} {}: dictionary {:?}, generated {:?}",
                        member.source_row(),
                        member.canonical_lemma(),
                        row.feature,
                        row.form,
                        generated
                    ));
                }
            }
        }
        let divergence_digest = format!("{:x}", Sha256::digest(mismatches.join("\n")));
        // The 518 divergences are retained, not discarded: public exact-table
        // precedence exposes the pinned Kaikki spellings, while the reviewed
        // source identity exposes Polivanova's canonical class prediction.
        // Hashing every row/cell/form tuple makes this complete comparison a
        // reproducible golden rather than accepting a sample or only a count.
        assert_eq!(compared, 6_114);
        assert_eq!(mismatches.len(), 518);
        assert_eq!(
            divergence_digest,
            "4679ba61e97dfa2da74f3881a3232525938518fd2a3a2d1e8146cca24b2b219c"
        );
    }

    #[test]
    fn noun_deformation_partition_is_explicit() {
        assert_eq!(
            osd_noun_deformation_route("2/m*"),
            Some("polivanova-agent-noun-deformation")
        );
        assert_eq!(
            osd_noun_deformation_route("2/m++"),
            Some("polivanova-in-noun-deformation")
        );
        assert_eq!(
            osd_noun_deformation_route("2/f*"),
            Some("polivanova-feminine-i-deformation")
        );
        assert_eq!(osd_noun_deformation_route("2/m"), None);
        let in_member = TwofoldNounFamilyMember::classify_source_lemma("гражданинъ")
            .expect("reviewed in-stem member");
        assert!(noun_deformation_class_matches(in_member, "2/m++"));
    }

    #[test]
    fn unique_nominal_ownership_is_exhaustive_and_cross_pos_explicit() {
        assert_eq!(
            unique_nominal_owner("имѧ", "0/n"),
            Some("polivanova-unique-noun-family")
        );
        assert_eq!(
            unique_nominal_owner("господь", "0/m"),
            Some("polivanova-unique-noun-family")
        );
        assert_eq!(
            unique_nominal_owner("десѧть", "0/m"),
            Some("reviewed-cardinal-ten")
        );
        for lemma in ["азъ", "ты", "сѧ"] {
            assert_eq!(
                unique_nominal_owner(lemma, "0/s"),
                Some("reviewed-personal-reflexive-pronoun")
            );
        }
        assert_eq!(unique_nominal_owner("неизвѣстъ", "0/m"), None);
    }

    #[test]
    fn final_classifications_and_support_states_are_closed() {
        assert_eq!(CLASSIFICATIONS.len(), 7);
        assert_eq!(SUPPORT_STATES.len(), 5);
        assert!(!CLASSIFICATIONS.contains(&"unclassified"));
    }

    #[test]
    fn latin_or_mixed_lemmas_do_not_get_fabricated_keys() {
        assert_eq!(
            old_church_slavonic_core::orthography::detect_script("slovo"),
            old_church_slavonic_core::Script::Latin
        );
        assert_eq!(safe_key("slovo"), "-");
    }
}
