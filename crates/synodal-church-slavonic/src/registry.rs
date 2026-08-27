use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope,
    ActiveParticipleShortFormation, AdjectiveClass, AdjectiveForm, AdjectiveLexeme, Animacy,
    AoristFormation, Aspect, AuthorityRole, BreathingMark, BreathingRule, Case, Comparison,
    ComparisonFormation, Confidence, DeterminerDeclension, DeterminerLexeme, EpistemicRole, Error,
    Evidence, EvidenceId, EvidenceKind, FiniteTense, Gender, GenerationPolicy, GrammarCell,
    ImperativeFormation, ImperfectFormation, InitialPresentation, LetterOccurrence, LexemeId,
    NounAnimacyInventory, NounDeclension, NounLexeme, NounNumberInventory, Number,
    NumeralDeclension, NumeralLexeme, ParticiplePrincipalPart, ParticipleTense, ParticipleVoice,
    PositionalOperation, PositionalParadigm, PositionalReplacement, PositionalRule,
    PronounDeclension, PronounEnvironment, PronounFormSelection, PronounLexeme,
    PronounPostpositive, PronounPrefix, Recension, RecensionMappingId, Result,
    ShortMasculineStemFormation, SourceId, SynodalWord, VerbConjugation, VerbLexeme,
    VerbalNounPrincipalPart, normalize_lookup_accentless, validate_adjective_lexeme,
    validate_determiner_lexeme, validate_numeral_lexeme, validate_pronoun_lexeme,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawLexeme(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawNounRestriction(pub [&'static str; 5]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPrincipalPart(pub [&'static str; 6]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawExactForm(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAlignment(pub [&'static str; 11]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAbbreviation(pub [&'static str; 13]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAbbreviationFamily(pub [&'static str; 12]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAccent(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAccentParadigm(pub [&'static str; 11]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPositionalRule(pub [&'static str; 7]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPositionalParadigm(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawTransformationRule(pub [&'static str; 6]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawConflict(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawDefectiveInventory(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawIrregularVerbInventory(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawReviewedEvidence(pub [&'static str; 6]);

/// The generated registry artifact: `generated/registry.dat`, written by the
/// extractor from `data/synodal/*.tsv`. It is a line-oriented text file —
/// `@TABLE <columns>` headers followed by tab-separated rows — embedded at
/// compile time and parsed once on first use. Keeping the data out of Rust
/// source means an admission changes a data file, not a 12k-line literal.
const EMBEDDED_ARTIFACT: &str = include_str!("../generated/registry.dat");

/// Build-time fingerprint of `generated/registry.dat` (FNV-1a over the raw
/// bytes, plus the byte length), computed by `build.rs`.
pub(crate) const EMBEDDED_FINGERPRINT: &str = env!("SYNODAL_REGISTRY_FINGERPRINT");

/// FNV-1a over the artifact bytes plus the byte length — the same function
/// `build.rs` uses, so a runtime consumer can fingerprint an artifact on disk
/// and compare it with [`EMBEDDED_FINGERPRINT`] or an installed override.
#[must_use]
pub fn registry_fingerprint(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}-{}", bytes.len())
}

/// Every generated table, parsed from one artifact. Tables keyed by lexeme
/// are emitted sorted by their first column (see `rows_for`).
pub(crate) struct Tables {
    pub(crate) fingerprint: String,
    pub(crate) lexemes: Vec<RawLexeme>,
    pub(crate) noun_restrictions: Vec<RawNounRestriction>,
    pub(crate) principal_parts: Vec<RawPrincipalPart>,
    pub(crate) exact_forms: Vec<RawExactForm>,
    pub(crate) alignments: Vec<RawAlignment>,
    pub(crate) abbreviations: Vec<RawAbbreviation>,
    pub(crate) abbreviation_families: Vec<RawAbbreviationFamily>,
    pub(crate) accents: Vec<RawAccent>,
    pub(crate) accent_paradigms: Vec<RawAccentParadigm>,
    pub(crate) positional_paradigms: Vec<RawPositionalParadigm>,
    pub(crate) positional_rules: Vec<RawPositionalRule>,
    pub(crate) transformation_rules: Vec<RawTransformationRule>,
    pub(crate) conflicts: Vec<RawConflict>,
    pub(crate) defective_inventories: Vec<RawDefectiveInventory>,
    pub(crate) irregular_verb_inventory: Vec<RawIrregularVerbInventory>,
    pub(crate) reviewed_evidence: Vec<RawReviewedEvidence>,
}

fn parse_rows<const N: usize, T>(
    name: &str,
    sections: &std::collections::BTreeMap<&str, (usize, Vec<&'static str>)>,
    wrap: fn([&'static str; N]) -> T,
) -> std::result::Result<Vec<T>, String> {
    let (columns, lines) = sections
        .get(name)
        .ok_or_else(|| format!("registry artifact lacks table {name}"))?;
    if *columns != N {
        return Err(format!(
            "registry artifact table {name} declares {columns} columns, the crate expects {N}"
        ));
    }
    lines
        .iter()
        .map(|line| {
            let fields: Vec<&'static str> = line.split('\t').collect();
            <[&'static str; N]>::try_from(fields)
                .map(wrap)
                .map_err(|fields| {
                    format!(
                        "registry artifact table {name} row has {} fields, expected {N}",
                        fields.len()
                    )
                })
        })
        .collect()
}

/// Parses one artifact. The text must live for `'static` because the parsed
/// rows borrow from it (zero-copy for the embedded artifact).
pub(crate) fn parse_tables(text: &'static str) -> std::result::Result<Tables, String> {
    let mut sections: std::collections::BTreeMap<&str, (usize, Vec<&'static str>)> =
        std::collections::BTreeMap::new();
    let mut current: Option<&str> = None;
    for line in text.lines() {
        if let Some(header) = line.strip_prefix('@') {
            let (name, columns) = header
                .split_once(' ')
                .ok_or_else(|| format!("malformed registry table header {line:?}"))?;
            let columns = columns
                .parse::<usize>()
                .map_err(|_| format!("malformed registry table header {line:?}"))?;
            if sections.insert(name, (columns, Vec::new())).is_some() {
                return Err(format!("registry artifact repeats table {name}"));
            }
            current = Some(name);
        } else if line.is_empty() || (line.starts_with('#') && current.is_none()) {
            continue;
        } else if let Some(name) = current {
            if let Some((_, lines)) = sections.get_mut(name) {
                lines.push(line);
            }
        } else {
            return Err(format!(
                "registry artifact row precedes any table header: {line:?}"
            ));
        }
    }
    Ok(Tables {
        fingerprint: registry_fingerprint(text.as_bytes()),
        lexemes: parse_rows("LEXEMES", &sections, RawLexeme)?,
        noun_restrictions: parse_rows("NOUN_RESTRICTIONS", &sections, RawNounRestriction)?,
        principal_parts: parse_rows("PRINCIPAL_PARTS", &sections, RawPrincipalPart)?,
        exact_forms: parse_rows("EXACT_FORMS", &sections, RawExactForm)?,
        alignments: parse_rows("ALIGNMENTS", &sections, RawAlignment)?,
        abbreviations: parse_rows("ABBREVIATIONS", &sections, RawAbbreviation)?,
        abbreviation_families: parse_rows(
            "ABBREVIATION_FAMILIES",
            &sections,
            RawAbbreviationFamily,
        )?,
        accents: parse_rows("ACCENTS", &sections, RawAccent)?,
        accent_paradigms: parse_rows("ACCENT_PARADIGMS", &sections, RawAccentParadigm)?,
        positional_paradigms: parse_rows("POSITIONAL_PARADIGMS", &sections, RawPositionalParadigm)?,
        positional_rules: parse_rows("POSITIONAL_RULES", &sections, RawPositionalRule)?,
        transformation_rules: parse_rows("TRANSFORMATION_RULES", &sections, RawTransformationRule)?,
        conflicts: parse_rows("CONFLICTS", &sections, RawConflict)?,
        defective_inventories: parse_rows(
            "DEFECTIVE_INVENTORIES",
            &sections,
            RawDefectiveInventory,
        )?,
        irregular_verb_inventory: parse_rows(
            "IRREGULAR_VERB_INVENTORY",
            &sections,
            RawIrregularVerbInventory,
        )?,
        reviewed_evidence: parse_rows("REVIEWED_EVIDENCE", &sections, RawReviewedEvidence)?,
    })
}

static EMBEDDED: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();

#[cfg(feature = "registry-override")]
static OVERRIDE: std::sync::RwLock<Option<&'static Tables>> = std::sync::RwLock::new(None);
/// Set once an override is installed so the hot path pays one atomic load,
/// not a lock acquisition, per lookup until then.
#[cfg(feature = "registry-override")]
static OVERRIDE_INSTALLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(feature = "registry-override")]
fn installed_override() -> Option<&'static Tables> {
    if !OVERRIDE_INSTALLED.load(std::sync::atomic::Ordering::Acquire) {
        return None;
    }
    OVERRIDE.read().ok().and_then(|guard| *guard)
}

/// The active registry: the development override when one is installed,
/// otherwise the embedded artifact. The embedded artifact is generated and
/// validated by the extractor and pinned by `embedded_artifact_parses`, so a
/// parse failure here is a build defect, not a runtime condition.
pub(crate) fn tables() -> &'static Tables {
    #[cfg(feature = "registry-override")]
    if let Some(installed) = installed_override() {
        return installed;
    }
    EMBEDDED.get_or_init(|| match parse_tables(EMBEDDED_ARTIFACT) {
        Ok(tables) => tables,
        Err(reason) => panic!("embedded generated/registry.dat is malformed: {reason}"),
    })
}

/// Fingerprint of the registry every lookup currently reads: the installed
/// override's when one is active, otherwise the embedded artifact's.
#[must_use]
pub fn active_registry_fingerprint() -> &'static str {
    #[cfg(feature = "registry-override")]
    if let Some(installed) = installed_override() {
        return installed.fingerprint.as_str();
    }
    EMBEDDED_FINGERPRINT
}

/// Development-only registry swap for the workspace tooling (`xtask`): the
/// admission inner loop regenerates the artifact and installs it in-process
/// instead of recompiling the crate. Not part of the published API — the
/// feature is off by default and the published crate exposes no override
/// channel. The installed text is leaked (rows borrow from it for `'static`);
/// callers install a handful of artifacts per process. Any analyzer or index
/// built before the install keeps reading the registry it was built from.
#[cfg(feature = "registry-override")]
pub fn install_registry_override(text: String) -> Result<&'static str> {
    let text: &'static str = Box::leak(text.into_boxed_str());
    let tables = parse_tables(text).map_err(|reason| Error::ContradictoryMetadata { reason })?;
    let tables: &'static Tables = Box::leak(Box::new(tables));
    let mut guard = OVERRIDE.write().map_err(|_| Error::ContradictoryMetadata {
        reason: "registry override lock poisoned".into(),
    })?;
    *guard = Some(tables);
    OVERRIDE_INSTALLED.store(true, std::sync::atomic::Ordering::Release);
    Ok(tables.fingerprint.as_str())
}

mod lexemes;
mod lookup;
mod types;

pub use types::*;

pub(crate) use lexemes::*;
pub(crate) use lookup::*;

#[cfg(test)]
mod artifact_tests {
    use super::*;

    #[test]
    fn embedded_artifact_parses_and_matches_the_build_fingerprint() {
        let tables = parse_tables(EMBEDDED_ARTIFACT).expect("embedded artifact parses");
        assert!(!tables.lexemes.is_empty());
        assert_eq!(tables.fingerprint, EMBEDDED_FINGERPRINT);
        assert_eq!(active_registry_fingerprint(), EMBEDDED_FINGERPRINT);
    }

    #[test]
    fn artifact_parser_rejects_shape_errors() {
        let missing = parse_tables("# comment\n@LEXEMES 9\n");
        assert!(missing.is_err(), "missing tables must be rejected");
        let short = parse_tables(concat!(
            "@LEXEMES 9\na\tb\n@NOUN_RESTRICTIONS 5\n@PRINCIPAL_PARTS 6\n@EXACT_FORMS 9\n",
            "@ALIGNMENTS 11\n@ABBREVIATIONS 13\n@ABBREVIATION_FAMILIES 12\n@ACCENTS 8\n",
            "@ACCENT_PARADIGMS 11\n@POSITIONAL_PARADIGMS 9\n@POSITIONAL_RULES 7\n",
            "@TRANSFORMATION_RULES 6\n@CONFLICTS 8\n@DEFECTIVE_INVENTORIES 8\n",
            "@IRREGULAR_VERB_INVENTORY 8\n@REVIEWED_EVIDENCE 6\n"
        ));
        assert!(short.is_err(), "a short row must be rejected");
    }
}
