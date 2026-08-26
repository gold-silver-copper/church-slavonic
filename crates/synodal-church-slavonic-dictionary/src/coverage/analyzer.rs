#[allow(unused_imports)]
use super::*;

/// Reusable reverse-analysis index. Building it is deliberately explicit so a
/// corpus run pays the paradigm-enumeration cost once rather than once per
/// token.
#[derive(Clone, Debug)]
pub struct Analyzer {
    pub(crate) inflector: Inflector,
    pub(crate) indexed_cells: usize,
    pub(crate) expanded_marked: BTreeMap<String, Vec<Analysis>>,
    pub(crate) expanded: BTreeMap<String, Vec<Analysis>>,
    pub(crate) printed_marked: BTreeMap<String, Vec<Analysis>>,
    pub(crate) printed: BTreeMap<String, Vec<Analysis>>,
    pub(crate) cardinal_expanded_marked: BTreeMap<String, Vec<CardinalWordAnalysis>>,
    pub(crate) cardinal_expanded: BTreeMap<String, Vec<CardinalWordAnalysis>>,
    pub(crate) cardinal_printed_marked: BTreeMap<String, Vec<CardinalWordAnalysis>>,
    pub(crate) cardinal_printed: BTreeMap<String, Vec<CardinalWordAnalysis>>,
    pub(crate) spelling_candidates: BTreeMap<String, BTreeSet<LexemeId>>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct AnalyzerConfig {
    generation_policy: GenerationPolicy,
    orthography: OrthographyProfile,
    productive_mapping_threshold_basis_points: u16,
}

impl From<Inflector> for AnalyzerConfig {
    fn from(inflector: Inflector) -> Self {
        Self {
            generation_policy: inflector.generation_policy(),
            orthography: inflector.orthography(),
            productive_mapping_threshold_basis_points: inflector
                .productive_mapping_threshold_basis_points(),
        }
    }
}

/// Process-local cache for immutable analyzers. Callers choose its lifetime,
/// so custom configurations never leak into unrelated processes or tests.
#[derive(Debug, Default)]
pub struct AnalyzerCache {
    analyzers: Mutex<BTreeMap<AnalyzerConfig, Arc<Analyzer>>>,
    constructions: std::sync::atomic::AtomicUsize,
}

impl AnalyzerCache {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            analyzers: Mutex::new(BTreeMap::new()),
            constructions: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn get(&self, inflector: Inflector) -> Result<Arc<Analyzer>> {
        let key = AnalyzerConfig::from(inflector);
        let mut analyzers = match self.analyzers.lock() {
            Ok(analyzers) => analyzers,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(analyzer) = analyzers.get(&key) {
            return Ok(Arc::clone(analyzer));
        }
        let analyzer = Arc::new(Analyzer::new(inflector)?);
        self.constructions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        analyzers.insert(key, Arc::clone(&analyzer));
        Ok(analyzer)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self.analyzers.lock() {
            Ok(analyzers) => analyzers.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of successful analyzer constructions performed by this cache.
    #[must_use]
    pub fn construction_count(&self) -> usize {
        self.constructions
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub(crate) static DEFAULT_ANALYZERS: OnceLock<AnalyzerCache> = OnceLock::new();

pub(crate) fn default_analyzer() -> Result<Arc<Analyzer>> {
    DEFAULT_ANALYZERS
        .get_or_init(AnalyzerCache::new)
        .get(Inflector::default())
}

impl Analyzer {
    pub fn new(inflector: Inflector) -> Result<Self> {
        let mut analyzer = Self {
            inflector,
            indexed_cells: 0,
            expanded_marked: BTreeMap::new(),
            expanded: BTreeMap::new(),
            printed_marked: BTreeMap::new(),
            printed: BTreeMap::new(),
            cardinal_expanded_marked: BTreeMap::new(),
            cardinal_expanded: BTreeMap::new(),
            cardinal_printed_marked: BTreeMap::new(),
            cardinal_printed: BTreeMap::new(),
            spelling_candidates: BTreeMap::new(),
        };
        let expanded_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::Expanded)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        let printed_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::SynodalLiturgical)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        for lexeme in lexemes()? {
            analyzer
                .spelling_candidates
                .entry(spelling_key(lexeme.lemma()))
                .or_default()
                .insert(lexeme.id().clone());
            for cell in analysis_cells_for_lexeme(&lexeme, inflector)? {
                analyzer.indexed_cells += 1;
                analyzer.index_cell(&lexeme, cell, expanded_inflector);
                analyzer.index_cell(&lexeme, cell, printed_inflector);
            }
        }
        analyzer.index_cardinal_words(expanded_inflector);
        analyzer.index_cardinal_words(printed_inflector);
        for index in [
            &mut analyzer.expanded_marked,
            &mut analyzer.expanded,
            &mut analyzer.printed_marked,
            &mut analyzer.printed,
        ] {
            sort_index(index);
        }
        for index in [
            &mut analyzer.cardinal_expanded_marked,
            &mut analyzer.cardinal_expanded,
            &mut analyzer.cardinal_printed_marked,
            &mut analyzer.cardinal_printed,
        ] {
            sort_cardinal_word_index(index);
        }
        Ok(analyzer)
    }

    #[cfg(test)]
    pub(crate) fn new_exhaustive(inflector: Inflector) -> Result<Self> {
        let mut analyzer = Self {
            inflector,
            indexed_cells: 0,
            expanded_marked: BTreeMap::new(),
            expanded: BTreeMap::new(),
            printed_marked: BTreeMap::new(),
            printed: BTreeMap::new(),
            cardinal_expanded_marked: BTreeMap::new(),
            cardinal_expanded: BTreeMap::new(),
            cardinal_printed_marked: BTreeMap::new(),
            cardinal_printed: BTreeMap::new(),
            spelling_candidates: BTreeMap::new(),
        };
        let expanded_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::Expanded)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        let printed_inflector = Inflector::builder()
            .generation_policy(inflector.generation_policy())
            .orthography(OrthographyProfile::SynodalLiturgical)
            .productive_mapping_threshold_basis_points(
                inflector.productive_mapping_threshold_basis_points(),
            )
            .build();
        for lexeme in lexemes()? {
            analyzer
                .spelling_candidates
                .entry(spelling_key(lexeme.lemma()))
                .or_default()
                .insert(lexeme.id().clone());
            for cell in candidate_cells(lexeme.part_of_speech()) {
                analyzer.indexed_cells += 1;
                analyzer.index_cell(&lexeme, cell, expanded_inflector);
                analyzer.index_cell(&lexeme, cell, printed_inflector);
            }
        }
        analyzer.index_cardinal_words(expanded_inflector);
        analyzer.index_cardinal_words(printed_inflector);
        for index in [
            &mut analyzer.expanded_marked,
            &mut analyzer.expanded,
            &mut analyzer.printed_marked,
            &mut analyzer.printed,
        ] {
            sort_index(index);
        }
        for index in [
            &mut analyzer.cardinal_expanded_marked,
            &mut analyzer.cardinal_expanded,
            &mut analyzer.cardinal_printed_marked,
            &mut analyzer.cardinal_printed,
        ] {
            sort_cardinal_word_index(index);
        }
        Ok(analyzer)
    }

    #[must_use]
    pub const fn inflector(&self) -> Inflector {
        self.inflector
    }

    /// Number of per-lexeme typed cells admitted to this reverse index.
    #[must_use]
    pub const fn indexed_cell_count(&self) -> usize {
        self.indexed_cells
    }

    pub fn analyze(&self, word: &str) -> Result<Vec<Analysis>> {
        let mut analyses = self.analyze_profile(word, OrthographyProfile::Expanded)?;
        analyses.extend(self.analyze_profile(word, OrthographyProfile::SynodalLiturgical)?);
        deduplicate_analyses(&mut analyses);
        Ok(analyses)
    }

    /// Returns every typed one-token cardinal construction compatible with
    /// either expanded or liturgical presentation.
    pub fn analyze_cardinal_word(&self, word: &str) -> Result<Vec<CardinalWordAnalysis>> {
        let mut analyses =
            self.analyze_cardinal_word_profile(word, OrthographyProfile::Expanded)?;
        analyses.extend(
            self.analyze_cardinal_word_profile(word, OrthographyProfile::SynodalLiturgical)?,
        );
        deduplicate_cardinal_word_analyses(&mut analyses);
        Ok(analyses)
    }

    /// Returns typed fused-cardinal analyses under one explicit orthographic
    /// profile. Marked input is exact: accent-insensitive fallback is allowed
    /// only for genuinely unmarked input or the explicit accentless profile.
    pub fn analyze_cardinal_word_profile(
        &self,
        word: &str,
        profile: OrthographyProfile,
    ) -> Result<Vec<CardinalWordAnalysis>> {
        let parsed = SynodalWord::parse(word)?;
        let marked_key = normalize_lookup(parsed.canonical());
        let key = normalize_lookup_accentless(parsed.canonical());
        let allow_fallback = profile == OrthographyProfile::ExpandedAccentless || marked_key == key;
        let (marked, fallback) = match profile {
            OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                (&self.cardinal_expanded_marked, &self.cardinal_expanded)
            }
            OrthographyProfile::SynodalLiturgical => {
                (&self.cardinal_printed_marked, &self.cardinal_printed)
            }
        };
        let mut analyses = marked
            .get(&marked_key)
            .cloned()
            .filter(|analyses| !analyses.is_empty())
            .unwrap_or_else(|| {
                if allow_fallback {
                    fallback.get(&key).cloned().unwrap_or_default()
                } else {
                    Vec::new()
                }
            });
        deduplicate_cardinal_word_analyses(&mut analyses);
        Ok(analyses)
    }

    pub(crate) fn analyze_dictionary(&self, word: &str) -> Result<Vec<Analysis>> {
        let parsed = SynodalWord::parse(word)?;
        let marked_key = normalize_lookup(parsed.canonical());
        let key = normalize_lookup_accentless(parsed.canonical());
        let allow_fallback = marked_key == key
            || self.inflector.orthography() == OrthographyProfile::ExpandedAccentless;
        let mut analyses = self
            .expanded_marked
            .get(&marked_key)
            .into_iter()
            .flatten()
            .cloned()
            .chain(
                self.printed_marked
                    .get(&marked_key)
                    .into_iter()
                    .flatten()
                    .cloned(),
            )
            .collect::<Vec<_>>();
        let used_fallback = analyses.is_empty() && allow_fallback;
        if used_fallback {
            analyses.extend(self.expanded.get(&key).into_iter().flatten().cloned());
            analyses.extend(self.printed.get(&key).into_iter().flatten().cloned());
        }
        if let Ok(expansions) = crate::morphology::abbreviation::expand(parsed.canonical()) {
            if used_fallback && !expansions.is_empty() {
                analyses.clear();
            }
            for expansion in expansions {
                let lexeme = crate::morphology::advanced::lookup_by_id(&expansion.lexeme_id)?;
                analyses.push(Analysis {
                    lexeme,
                    cell: Some(expansion.cell),
                    matched_text: parsed.canonical().into(),
                    source: AnalysisSource::AbbreviationExpansion,
                    recension_mapping: None,
                    confidence: synodal_church_slavonic_core::Confidence::CERTAIN,
                    evidence_ids: expansion
                        .evidence_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    assumptions: Vec::new(),
                    contradictions: Vec::new(),
                    warnings: Vec::new(),
                    rule_trace: RuleTrace::default(),
                    reflexive: false,
                });
            }
        }
        let mut best_by_analysis = BTreeMap::new();
        for mut analysis in analyses {
            if used_fallback && analysis.source != AnalysisSource::AbbreviationExpansion {
                analysis
                    .warnings
                    .push("analysis required accent-insensitive matching".into());
            }
            let key = (
                analysis.lexeme.id().clone(),
                analysis.cell,
                analysis.source,
                analysis.recension_mapping.clone(),
            );
            best_by_analysis.entry(key).or_insert(analysis);
        }
        Ok(best_by_analysis.into_values().collect())
    }

    pub fn analyze_profile(
        &self,
        word: &str,
        profile: OrthographyProfile,
    ) -> Result<Vec<Analysis>> {
        let parsed = SynodalWord::parse(word)?;
        let marked_key = normalize_lookup(parsed.canonical());
        let key = normalize_lookup_accentless(parsed.canonical());
        let allow_fallback = profile == OrthographyProfile::ExpandedAccentless || marked_key == key;
        let (marked, fallback) = match profile {
            OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                (&self.expanded_marked, &self.expanded)
            }
            OrthographyProfile::SynodalLiturgical => (&self.printed_marked, &self.printed),
        };
        let lookup = |canonical: &str| -> Vec<Analysis> {
            let marked_key = normalize_lookup(canonical);
            let key = normalize_lookup_accentless(canonical);
            marked
                .get(&marked_key)
                .cloned()
                .filter(|analyses| !analyses.is_empty())
                .unwrap_or_else(|| {
                    if allow_fallback {
                        fallback.get(&key).cloned().unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                })
        };
        let mut analyses = lookup(parsed.canonical());
        if analyses.is_empty() {
            analyses = self.reflexive_analyses(parsed.canonical(), &lookup);
        }
        if let Ok(expansions) = crate::morphology::abbreviation::expand(parsed.canonical()) {
            for expansion in expansions {
                let lexeme = crate::morphology::advanced::lookup_by_id(&expansion.lexeme_id)?;
                analyses.push(Analysis {
                    lexeme,
                    cell: Some(expansion.cell),
                    matched_text: parsed.canonical().into(),
                    source: AnalysisSource::AbbreviationExpansion,
                    recension_mapping: None,
                    confidence: synodal_church_slavonic_core::Confidence::CERTAIN,
                    evidence_ids: expansion
                        .evidence_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    assumptions: Vec::new(),
                    contradictions: Vec::new(),
                    warnings: Vec::new(),
                    rule_trace: RuleTrace::default(),
                    reflexive: false,
                });
            }
        }
        deduplicate_analyses(&mut analyses);
        Ok(analyses)
    }

    /// Alypy §73: a surface in `-сѧ` with no registered reading is analysed
    /// as the reflexive/passive voice of a registered active verb when the
    /// host it was built from (with the deleted final jer restored where the
    /// rule removed one) has a verbal reading. The lexeme and cell are the
    /// host's; the analysis is marked `reflexive` and carries the host in its
    /// trace, and its source is the rule, not the host's evidence row, because
    /// no row cites this surface.
    pub(crate) fn reflexive_analyses(
        &self,
        canonical: &str,
        lookup: &dyn Fn(&str) -> Vec<Analysis>,
    ) -> Vec<Analysis> {
        for host in synodal_church_slavonic_core::reflexive_base_candidates(canonical) {
            let derived: Vec<Analysis> = lookup(&host)
                .into_iter()
                .filter(|analysis| {
                    matches!(
                        analysis.cell,
                        Some(
                            GrammarCell::FiniteVerb(_)
                                | GrammarCell::Imperative(_)
                                | GrammarCell::Infinitive
                                | GrammarCell::LParticiple(_)
                                | GrammarCell::Participle(_)
                        )
                    ) && !analysis.lexeme.lemma().ends_with("сѧ")
                        && !analysis.reflexive
                })
                .map(|mut analysis| {
                    let mut trace = analysis.rule_trace.clone();
                    trace.push(TraceStep {
                        rule: RuleId::from(synodal_church_slavonic_core::REFLEXIVE_RULE_ID),
                        stage: "reflexive-enclitic".into(),
                        input: host.clone(),
                        output: canonical.to_owned(),
                        source_recension: Some(Recension::SynodalRussian),
                        target_recension: Recension::SynodalRussian,
                        mapping: None,
                        evidence: vec![],
                    });
                    analysis.rule_trace = trace;
                    analysis.matched_text = canonical.to_owned();
                    analysis.source = AnalysisSource::SynodalProductiveRule;
                    analysis.assumptions.push(format!(
                        "reflexive or passive voice of {} by Alypy §73: host form {host} + enclitic сѧ",
                        analysis.lexeme.lemma()
                    ));
                    analysis.reflexive = true;
                    analysis
                })
                .collect();
            if !derived.is_empty() {
                return derived;
            }
        }
        Vec::new()
    }

    #[must_use]
    pub fn spelling_candidates(&self, word: &str) -> Vec<LexemeId> {
        self.spelling_candidates
            .get(&spelling_key(word))
            .map_or_else(Vec::new, |ids| ids.iter().cloned().collect())
    }

    pub(crate) fn index_cardinal_words(&mut self, inflector: Inflector) {
        const VALUES: [u32; 26] = [
            11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300, 400,
            500, 600, 700, 800, 900,
        ];
        let profile = inflector.orthography();
        for value in VALUES {
            for case in Case::ALL.into_iter().filter(|case| *case != Case::Vocative) {
                for animacy in Animacy::ALL {
                    let genders: &[Option<Gender>] = if (11..=14).contains(&value) {
                        &[
                            Some(Gender::Masculine),
                            Some(Gender::Feminine),
                            Some(Gender::Neuter),
                        ]
                    } else {
                        &[None]
                    };
                    for gender in genders {
                        let cell = CompoundNumeralCell {
                            case,
                            gender: *gender,
                            animacy,
                        };
                        let Ok(realized) = cardinal_with(value, cell, inflector) else {
                            continue;
                        };
                        for phrase in realized
                            .analyses()
                            .iter()
                            .filter(|phrase| phrase.tokens.len() == 1)
                        {
                            let Some(token) = phrase.tokens.first() else {
                                continue;
                            };
                            for variant in token.forms.variants() {
                                let surface = match profile {
                                    OrthographyProfile::Expanded
                                    | OrthographyProfile::ExpandedAccentless => &variant.expanded,
                                    OrthographyProfile::SynodalLiturgical => &variant.printed,
                                };
                                let Ok(canonical) = SynodalWord::parse(surface) else {
                                    continue;
                                };
                                let analysis = CardinalWordAnalysis {
                                    value,
                                    cell,
                                    construction: phrase.construction,
                                    matched_text: surface.clone(),
                                    source: analysis_source(&variant.source),
                                    confidence: variant.confidence,
                                    evidence_ids: variant
                                        .evidence
                                        .iter()
                                        .map(|evidence| evidence.id.to_string())
                                        .collect(),
                                    assumptions: variant
                                        .assumptions
                                        .iter()
                                        .map(|assumption| assumption.detail.clone())
                                        .collect(),
                                    contradictions: variant
                                        .contradictions
                                        .iter()
                                        .map(|contradiction| contradiction.detail.clone())
                                        .collect(),
                                    warnings: variant.warnings.clone(),
                                    rule_trace: variant.rule_trace.clone(),
                                };
                                let marked_key = normalize_lookup(canonical.canonical());
                                let key = normalize_lookup_accentless(canonical.canonical());
                                let (marked, fallback) = match profile {
                                    OrthographyProfile::Expanded
                                    | OrthographyProfile::ExpandedAccentless => (
                                        &mut self.cardinal_expanded_marked,
                                        &mut self.cardinal_expanded,
                                    ),
                                    OrthographyProfile::SynodalLiturgical => (
                                        &mut self.cardinal_printed_marked,
                                        &mut self.cardinal_printed,
                                    ),
                                };
                                marked.entry(marked_key).or_default().push(analysis.clone());
                                fallback.entry(key).or_default().push(analysis);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn index_cell(&mut self, lexeme: &LexemeSummary, cell: GrammarCell, inflector: Inflector) {
        let Ok(forms) = inflector.form_by_id(lexeme.id(), cell) else {
            return;
        };
        let profile = inflector.orthography();
        for variant in forms.variants() {
            let surface = match profile {
                OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                    &variant.expanded
                }
                OrthographyProfile::SynodalLiturgical => &variant.printed,
            };
            let Ok(canonical_surface) = SynodalWord::parse(surface) else {
                continue;
            };
            let marked_key = normalize_lookup(canonical_surface.canonical());
            let key = normalize_lookup_accentless(canonical_surface.canonical());
            let analysis = Analysis {
                lexeme: lexeme.clone(),
                cell: Some(cell),
                matched_text: surface.clone(),
                source: analysis_source(&variant.source),
                recension_mapping: variant.recension_mapping.clone(),
                confidence: variant.confidence,
                evidence_ids: variant
                    .evidence
                    .iter()
                    .map(|evidence| evidence.id.to_string())
                    .collect(),
                assumptions: variant
                    .assumptions
                    .iter()
                    .map(|assumption| assumption.detail.clone())
                    .collect(),
                contradictions: variant
                    .contradictions
                    .iter()
                    .map(|contradiction| contradiction.detail.clone())
                    .collect(),
                warnings: variant.warnings.clone(),
                rule_trace: variant.rule_trace.clone(),
                reflexive: false,
            };
            let (marked, fallback) = match profile {
                OrthographyProfile::Expanded | OrthographyProfile::ExpandedAccentless => {
                    (&mut self.expanded_marked, &mut self.expanded)
                }
                OrthographyProfile::SynodalLiturgical => {
                    (&mut self.printed_marked, &mut self.printed)
                }
            };
            marked.entry(marked_key).or_default().push(analysis.clone());
            fallback.entry(key).or_default().push(analysis);
        }
    }
}

pub fn check_text(analyzer: &Analyzer, text: &str, options: CheckTextOptions) -> TextReport {
    let mut analyses = Vec::new();
    let mut unique = BTreeSet::new();
    let mut summary = TextSummary::default();
    for token in tokenize(text) {
        unique.insert(token.normalized.clone());
        let analysis = classify_token(analyzer, token, &options);
        update_text_summary(&mut summary, &analysis);
        analyses.push(analysis);
    }
    summary.total_tokens = analyses.len();
    summary.unique_tokens = unique.len();
    TextReport {
        schema_version: 1,
        options,
        summary,
        tokens: analyses,
    }
}
