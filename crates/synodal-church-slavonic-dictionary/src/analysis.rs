#[allow(unused_imports)]
use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AnalysisSource {
    ExactSynodalAttestation,
    SynodalIrregularOverride,
    SynodalNormativeTable,
    SynodalProductiveRule,
    CallerSpecifiedPrediction,
    InheritedPrediction,
    AnalogicalPrediction,
    AbbreviationExpansion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub lexeme: LexemeSummary,
    pub cell: Option<GrammarCell>,
    pub matched_text: String,
    pub source: AnalysisSource,
    pub recension_mapping: Option<RecensionMappingId>,
    pub confidence: Confidence,
    pub evidence_ids: Vec<String>,
    pub assumptions: Vec<String>,
    pub contradictions: Vec<String>,
    pub warnings: Vec<String>,
    pub rule_trace: RuleTrace,
    /// The surface is a reflexive/passive form (Alypy §73) derived by rule
    /// from a registered active verb: the lexeme and cell describe the host
    /// the enclitic `-сѧ` attached to.
    #[serde(default)]
    pub reflexive: bool,
}

/// Build-time fingerprint of `generated/registry.rs` (FNV-1a over the raw
/// bytes, plus the byte length). The xtask staleness tripwire compares this
/// against the on-disk file so a stale binary refuses to measure.
pub const REGISTRY_FINGERPRINT: &str = env!("SYNODAL_REGISTRY_FINGERPRINT");

/// Returns every compatible curated analysis of an expanded or printed word.
pub fn analyze(word: &str) -> Result<Vec<Analysis>> {
    coverage::default_analyzer()?.analyze_dictionary(word)
}

/// Returns every compatible curated analysis admitted by the caller's
/// generation and orthography policy. The default `analyze` remains Strict;
/// callers must opt into inherited or exploratory predictions explicitly.
pub fn analyze_with(word: &str, inflector: Inflector) -> Result<Vec<Analysis>> {
    if inflector == Inflector::default() {
        return analyze(word);
    }
    coverage::Analyzer::new(inflector)?.analyze_dictionary(word)
}

/// One token of a passage with everything a consumer needs: its reviewed
/// readings in provenance order (attested and normative before predictions of
/// any kind), and — only under [`GenerationPolicy::Exploratory`] — the typed
/// segmentation hypotheses for a token that has no reviewed reading at all.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TokenReading {
    /// The token exactly as printed, with its byte span in the input.
    pub token: coverage::TextToken,
    /// Reviewed readings, best first. Every reading names its lexeme, cell,
    /// source (`is_attested`/`is_prediction` via [`AnalysisSource`]),
    /// confidence, evidence ids, and whether it is a §73 reflexive
    /// derivation.
    pub readings: Vec<Analysis>,
    /// Exploratory segmentation hypotheses for an unread token. Empty unless
    /// the inflector's policy is `Exploratory`; never mixed into `readings`.
    pub predictions: Vec<prediction::Prediction>,
}

/// A whole analysed passage with stable `serde` serialisation.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TextAnalysis {
    pub tokens: Vec<TokenReading>,
}

/// Analyses a passage the registry may never have seen: every token gets its
/// reviewed readings in provenance order, and under
/// [`GenerationPolicy::Exploratory`] an unread token additionally gets typed
/// segmentation hypotheses that are clearly separated from reviewed readings.
///
/// ```
/// use synodal_church_slavonic_dictionary::analyze_text;
/// use synodal_church_slavonic::Inflector;
///
/// // Acts 8:30 in the held-out evaluation partition — never a source of
/// // reviewed evidence.
/// let passage = "ᲂу҆слы́ша є҆го̀ чтꙋ́ща прⷪ҇ро́ка";
/// let analysis = analyze_text(passage, Inflector::default())?;
/// let first = &analysis.tokens[0];
/// assert_eq!(first.token.original, "ᲂу҆слы́ша");
/// let reading = &first.readings[0];
/// assert_eq!(reading.lexeme.lemma(), "оуслышати");
/// assert!(matches!(
///     reading.source,
///     synodal_church_slavonic_dictionary::AnalysisSource::SynodalProductiveRule
/// ));
/// # Ok::<(), synodal_church_slavonic::Error>(())
/// ```
pub fn analyze_text(text: &str, inflector: Inflector) -> Result<TextAnalysis> {
    let analyzer = if inflector == Inflector::default() {
        coverage::default_analyzer()?
    } else {
        std::sync::Arc::new(coverage::Analyzer::new(inflector)?)
    };
    let report = coverage::check_text(
        &analyzer,
        text,
        coverage::CheckTextOptions {
            generation_policy: inflector.generation_policy(),
            orthography_profile: inflector.orthography(),
        },
    );
    let tokens = report
        .tokens
        .into_iter()
        .map(|analysis| {
            let predictions = if analysis.analyses.is_empty() {
                prediction::predict_under(inflector.generation_policy(), &analysis.token.normalized)
            } else {
                Vec::new()
            };
            TokenReading {
                token: analysis.token,
                readings: analysis.analyses,
                predictions,
            }
        })
        .collect();
    Ok(TextAnalysis { tokens })
}

/// Returns typed analyses of a fused cardinal word without fabricating a
/// dictionary lexeme for the grammatical construction.
pub fn analyze_cardinal_word(word: &str) -> Result<Vec<coverage::CardinalWordAnalysis>> {
    coverage::default_analyzer()?.analyze_cardinal_word(word)
}

/// Returns typed fused-cardinal analyses under a caller-selected policy.
pub fn analyze_cardinal_word_with(
    word: &str,
    inflector: Inflector,
) -> Result<Vec<coverage::CardinalWordAnalysis>> {
    if inflector == Inflector::default() {
        return analyze_cardinal_word(word);
    }
    coverage::Analyzer::new(inflector)?.analyze_cardinal_word(word)
}

pub fn lemmatize(word: &str) -> Result<Vec<Entry>> {
    lemmatize_with(word, Inflector::default())
}

pub fn lemmatize_with(word: &str, inflector: Inflector) -> Result<Vec<Entry>> {
    let ids: BTreeSet<LexemeId> = analyze_with(word, inflector)?
        .into_iter()
        .map(|analysis| analysis.lexeme.id().clone())
        .collect();
    ids.iter().map(lookup_by_id).collect()
}
