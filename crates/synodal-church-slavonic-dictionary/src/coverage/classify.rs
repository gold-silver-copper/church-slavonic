#[allow(unused_imports)]
use super::*;

pub(crate) fn classify_token(
    analyzer: &Analyzer,
    token: TextToken,
    options: &CheckTextOptions,
) -> TextTokenAnalysis {
    if let Ok(numeral) = CyrillicNumeral::parse(&token.original) {
        return TextTokenAnalysis {
            token,
            status: TokenStatus::CyrillicNumeral,
            analyses: Vec::new(),
            numeral: Some(numeral),
            cardinal_words: Vec::new(),
            gap: None,
        };
    }
    if let Err(error) = SynodalWord::parse(&token.original) {
        let kind = if contains_cyrillic(&token.original)
            && !contains_non_cyrillic_alphabetic(&token.original)
        {
            GapKind::MissingAccentOrOrthographicMetadata
        } else {
            GapKind::UnknownLexeme
        };
        return TextTokenAnalysis {
            token,
            status: TokenStatus::Unresolved,
            analyses: Vec::new(),
            numeral: None,
            cardinal_words: Vec::new(),
            gap: Some(GapOccurrence {
                kind,
                secondary_reasons: Vec::new(),
                detail: error.to_string(),
                candidate_lexeme_ids: Vec::new(),
                requested_morphological_system: None,
                missing_metadata: Vec::new(),
                resolver_trace: RuleTrace::default(),
                suggested_action: if kind == GapKind::UnknownLexeme {
                    "replace Latin or later-language fallback text, or review a new target lexeme"
                        .into()
                } else {
                    "review malformed combining marks, titlo expansion, accents, and positional spelling"
                        .into()
                },
            }),
        };
    }
    let cardinal_words = analyzer
        .analyze_cardinal_word_profile(&token.original, options.orthography_profile)
        .unwrap_or_default();
    let analyses = analyzer
        .analyze_profile(&token.original, options.orthography_profile)
        .unwrap_or_default();
    if !analyses.is_empty() {
        let ids: BTreeSet<&LexemeId> = analyses
            .iter()
            .map(|analysis| analysis.lexeme.id())
            .collect();
        if ids.len() > 1 {
            return TextTokenAnalysis {
                gap: Some(ambiguity_gap(
                    &analyses,
                    "several target lexemes match this surface",
                )),
                token,
                status: TokenStatus::Ambiguous,
                analyses,
                numeral: None,
                cardinal_words,
            };
        }
        let status = analyses
            .iter()
            .map(|analysis| status_for_source(analysis.source))
            .min()
            .unwrap_or(TokenStatus::Unresolved);
        return TextTokenAnalysis {
            token,
            status,
            analyses,
            numeral: None,
            cardinal_words,
            gap: None,
        };
    }

    if !cardinal_words.is_empty() {
        let values = cardinal_words
            .iter()
            .map(|analysis| analysis.value)
            .collect::<BTreeSet<_>>();
        let status = cardinal_words
            .iter()
            .min_by_key(|analysis| source_rank(analysis.source))
            .map_or(TokenStatus::Unresolved, |analysis| {
                status_for_source(analysis.source)
            });
        let gap = (values.len() > 1).then(|| cardinal_word_ambiguity_gap(&cardinal_words));
        return TextTokenAnalysis {
            token,
            status: if gap.is_some() {
                TokenStatus::Ambiguous
            } else {
                status
            },
            analyses: Vec::new(),
            numeral: None,
            cardinal_words,
            gap,
        };
    }

    if normalize_lookup(&token.original) != normalize_lookup_accentless(&token.original) {
        let accentless = analyzer
            .analyze_profile(&token.original, OrthographyProfile::ExpandedAccentless)
            .unwrap_or_default();
        if !accentless.is_empty() {
            return TextTokenAnalysis {
                token,
                status: TokenStatus::Unresolved,
                numeral: None,
                cardinal_words: Vec::new(),
                gap: Some(GapOccurrence {
                    kind: GapKind::MissingAccentOrOrthographicMetadata,
                    secondary_reasons: Vec::new(),
                    detail: "the accentless surface resolves, but the explicit presentation marks do not match reviewed evidence".into(),
                    candidate_lexeme_ids: analysis_ids(&accentless),
                    requested_morphological_system: accentless
                        .first()
                        .and_then(|analysis| analysis.cell)
                        .map(morphological_system),
                    missing_metadata: vec![MetadataField::AccentClass],
                    resolver_trace: accentless
                        .first()
                        .map_or_else(RuleTrace::default, |analysis| analysis.rule_trace.clone()),
                    suggested_action: "review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback".into(),
                }),
                analyses: accentless,
            };
        }
    }

    if options.orthography_profile == OrthographyProfile::SynodalLiturgical {
        let expanded = analyzer
            .analyze_profile(&token.original, OrthographyProfile::Expanded)
            .unwrap_or_default();
        if !expanded.is_empty() {
            let ids = analysis_ids(&expanded);
            return TextTokenAnalysis {
                token,
                status: TokenStatus::Unresolved,
                analyses: expanded,
                numeral: None,
                cardinal_words: Vec::new(),
                gap: Some(GapOccurrence {
                    kind: GapKind::MissingAccentOrOrthographicMetadata,
                    secondary_reasons: Vec::new(),
                    detail: "expanded morphology resolves, but the liturgical profile cannot realize this surface".into(),
                    candidate_lexeme_ids: ids,
                    requested_morphological_system: None,
                    missing_metadata: vec![MetadataField::AccentClass],
                    resolver_trace: RuleTrace::default(),
                    suggested_action: "review accent, breathing, and positional-letter evidence for the resolved lexeme".into(),
                }),
            };
        }
    }

    if let Ok(summary) = crate::morphology::lookup(&token.original) {
        let metadata = lexical_metadata(summary.id()).ok();
        let missing = missing_metadata_by_id(summary.id()).unwrap_or_default();
        let principal_missing: Vec<MetadataField> = missing
            .iter()
            .copied()
            .filter(|field| {
                matches!(
                    field,
                    MetadataField::PresentStem
                        | MetadataField::PresentFirstSingular
                        | MetadataField::PresentThirdPlural
                        | MetadataField::FutureStem
                        | MetadataField::FutureFirstSingular
                        | MetadataField::FutureThirdPlural
                        | MetadataField::ImperfectStem
                        | MetadataField::AoristStem
                        | MetadataField::ImperativeStem
                        | MetadataField::LParticipleStem
                        | MetadataField::ParticipleStem
                        | MetadataField::SupineStem
                        | MetadataField::VerbalNounStem
                )
            })
            .collect();
        let capabilities = capabilities_by_id(summary.id(), analyzer.inflector()).ok();
        let kind =
            if summary.part_of_speech() == PartOfSpeech::Verb && !principal_missing.is_empty() {
                GapKind::MissingVerbPrincipalPart
            } else if metadata
                .as_ref()
                .is_none_or(|metadata| metadata.class.is_none())
                || capabilities.as_ref().is_some_and(|capabilities| {
                    matches!(summary.part_of_speech(), PartOfSpeech::Noun)
                        && !capabilities.productive_noun
                        || matches!(summary.part_of_speech(), PartOfSpeech::Adjective)
                            && !capabilities.productive_adjective
                        || matches!(summary.part_of_speech(), PartOfSpeech::Determiner)
                            && !capabilities.productive_determiner
                        || matches!(summary.part_of_speech(), PartOfSpeech::Numeral)
                            && !capabilities.productive_numeral
                })
            {
                GapKind::MissingDeclensionOrClass
            } else {
                GapKind::UnsupportedFormation
            };
        let suggested_action = match kind {
            GapKind::MissingVerbPrincipalPart => {
                "review the independently sourced principal part required by the repeated corpus form"
            }
            GapKind::MissingDeclensionOrClass => {
                "review the target declension or lexical class before enabling generation"
            }
            _ => "identify the requested cell and add a cited Synodal rule or exact evidence",
        };
        return TextTokenAnalysis {
            token,
            status: TokenStatus::Unresolved,
            analyses: Vec::new(),
            numeral: None,
            cardinal_words: Vec::new(),
            gap: Some(GapOccurrence {
                kind,
                secondary_reasons: Vec::new(),
                detail: format!(
                    "known target lemma {} does not analyze in this context",
                    summary.id()
                ),
                candidate_lexeme_ids: vec![summary.id().clone()],
                requested_morphological_system: None,
                missing_metadata: if kind == GapKind::MissingVerbPrincipalPart {
                    principal_missing
                } else {
                    missing
                },
                resolver_trace: RuleTrace::default(),
                suggested_action: suggested_action.into(),
            }),
        };
    }

    let spelling_candidates = analyzer.spelling_candidates(&token.original);
    if !spelling_candidates.is_empty() {
        return TextTokenAnalysis {
            token,
            status: TokenStatus::SpellingVariant,
            analyses: Vec::new(),
            numeral: None,
            cardinal_words: Vec::new(),
            gap: Some(GapOccurrence {
                kind: GapKind::AmbiguityOrSpellingVariant,
                secondary_reasons: Vec::new(),
                detail: "the diagnostic spelling key matches one or more reviewed lemmas".into(),
                candidate_lexeme_ids: spelling_candidates,
                requested_morphological_system: None,
                missing_metadata: Vec::new(),
                resolver_trace: RuleTrace::default(),
                suggested_action: "review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme".into(),
            }),
        };
    }

    TextTokenAnalysis {
        token,
        status: TokenStatus::Unresolved,
        analyses: Vec::new(),
        numeral: None,
        cardinal_words: Vec::new(),
        gap: Some(GapOccurrence {
            kind: GapKind::UnknownLexeme,
            secondary_reasons: Vec::new(),
            detail: "no reviewed target lexeme or compatible generated form".into(),
            candidate_lexeme_ids: Vec::new(),
            requested_morphological_system: None,
            missing_metadata: Vec::new(),
            resolver_trace: RuleTrace::default(),
            suggested_action: "review the token against target-recension evidence and create or reject a lexical candidate".into(),
        }),
    }
}

#[must_use]
pub fn tokenize(text: &str) -> Vec<TextToken> {
    let mut tokens = Vec::new();
    let mut start = None::<(usize, usize, usize)>;
    let mut line = 1_usize;
    let mut column = 1_usize;
    for (byte, character) in text.char_indices() {
        let component = character.is_alphabetic()
            || is_combining_mark(character)
            || character == '\u{0482}'
            || ('\u{2de0}'..='\u{2dff}').contains(&character);
        if component {
            start.get_or_insert((byte, line, column));
        } else if let Some((byte_start, token_line, token_column)) = start.take() {
            push_token(
                &mut tokens,
                text,
                byte_start,
                byte,
                token_line,
                token_column,
            );
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    if let Some((byte_start, token_line, token_column)) = start {
        push_token(
            &mut tokens,
            text,
            byte_start,
            text.len(),
            token_line,
            token_column,
        );
    }
    tokens
}

pub(crate) fn push_token(
    tokens: &mut Vec<TextToken>,
    text: &str,
    byte_start: usize,
    byte_end: usize,
    line: usize,
    column: usize,
) {
    let original = &text[byte_start..byte_end];
    tokens.push(TextToken {
        original: original.into(),
        normalized: normalize_lookup_accentless(original),
        byte_start,
        byte_end,
        line,
        column,
    });
}

pub(crate) fn sort_index(index: &mut BTreeMap<String, Vec<Analysis>>) {
    for analyses in index.values_mut() {
        deduplicate_analyses(analyses);
    }
}

pub(crate) fn sort_cardinal_word_index(index: &mut BTreeMap<String, Vec<CardinalWordAnalysis>>) {
    for analyses in index.values_mut() {
        deduplicate_cardinal_word_analyses(analyses);
    }
}

pub(crate) fn deduplicate_cardinal_word_analyses(analyses: &mut Vec<CardinalWordAnalysis>) {
    analyses.sort_by(|left, right| {
        source_rank(left.source)
            .cmp(&source_rank(right.source))
            .then_with(|| left.value.cmp(&right.value))
            .then_with(|| left.cell.cmp(&right.cell))
            .then_with(|| left.construction.cmp(&right.construction))
            .then_with(|| left.matched_text.cmp(&right.matched_text))
    });
    analyses.dedup_by(|left, right| {
        left.value == right.value
            && left.cell == right.cell
            && left.construction == right.construction
            && left.source == right.source
            && left.matched_text == right.matched_text
    });
}

pub(crate) fn deduplicate_analyses(analyses: &mut Vec<Analysis>) {
    analyses.sort_by(|left, right| {
        source_rank(left.source)
            .cmp(&source_rank(right.source))
            .then_with(|| left.lexeme.id().cmp(right.lexeme.id()))
            .then_with(|| left.cell.cmp(&right.cell))
            .then_with(|| left.matched_text.cmp(&right.matched_text))
    });
    analyses.dedup_by(|left, right| {
        left.lexeme.id() == right.lexeme.id()
            && left.cell == right.cell
            && left.source == right.source
            && left.recension_mapping == right.recension_mapping
            && left.matched_text == right.matched_text
    });
}

pub(crate) const fn source_rank(source: AnalysisSource) -> u8 {
    match source {
        AnalysisSource::ExactSynodalAttestation => 0,
        AnalysisSource::SynodalIrregularOverride => 1,
        AnalysisSource::SynodalNormativeTable => 2,
        AnalysisSource::SynodalProductiveRule => 3,
        AnalysisSource::CallerSpecifiedPrediction => 4,
        AnalysisSource::AbbreviationExpansion => 5,
        AnalysisSource::InheritedPrediction => 6,
        AnalysisSource::AnalogicalPrediction => 7,
    }
}

pub(crate) const fn status_for_source(source: AnalysisSource) -> TokenStatus {
    match source {
        AnalysisSource::ExactSynodalAttestation => TokenStatus::ExactSynodalAttestation,
        AnalysisSource::SynodalIrregularOverride => TokenStatus::SynodalIrregularOverride,
        AnalysisSource::SynodalNormativeTable => TokenStatus::SynodalNormativeTable,
        AnalysisSource::SynodalProductiveRule => TokenStatus::SynodalProductiveRule,
        AnalysisSource::CallerSpecifiedPrediction => TokenStatus::CallerSpecifiedPrediction,
        AnalysisSource::InheritedPrediction => TokenStatus::InheritedPrediction,
        AnalysisSource::AnalogicalPrediction => TokenStatus::AnalogicalPrediction,
        AnalysisSource::AbbreviationExpansion => TokenStatus::AbbreviationExpansion,
    }
}

pub(crate) const fn status_label(status: TokenStatus) -> &'static str {
    match status {
        TokenStatus::ExactSynodalAttestation => "exact-synodal-attestation",
        TokenStatus::SynodalIrregularOverride => "synodal-irregular-override",
        TokenStatus::SynodalNormativeTable => "synodal-normative-table",
        TokenStatus::SynodalProductiveRule => "synodal-productive-rule",
        TokenStatus::CallerSpecifiedPrediction => "caller-specified-prediction",
        TokenStatus::InheritedPrediction => "inherited-prediction",
        TokenStatus::AnalogicalPrediction => "analogical-prediction",
        TokenStatus::AbbreviationExpansion => "abbreviation-expansion",
        TokenStatus::SpellingVariant => "spelling-variant",
        TokenStatus::Ambiguous => "ambiguous",
        TokenStatus::Unresolved => "unresolved",
        TokenStatus::CyrillicNumeral => "cyrillic-numeral",
    }
}

pub(crate) const fn policy_label(policy: GenerationPolicy) -> &'static str {
    match policy {
        GenerationPolicy::Strict => "strict",
        GenerationPolicy::Productive => "productive",
        GenerationPolicy::Exploratory => "exploratory",
    }
}

pub(crate) fn ambiguity_gap(analyses: &[Analysis], detail: &str) -> GapOccurrence {
    GapOccurrence {
        kind: GapKind::AmbiguityOrSpellingVariant,
        secondary_reasons: Vec::new(),
        detail: detail.into(),
        candidate_lexeme_ids: analysis_ids(analyses),
        requested_morphological_system: analyses
            .first()
            .and_then(|analysis| analysis.cell)
            .map(morphological_system),
        missing_metadata: Vec::new(),
        resolver_trace: analyses
            .first()
            .map_or_else(RuleTrace::default, |analysis| analysis.rule_trace.clone()),
        suggested_action: "review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains".into(),
    }
}

pub(crate) fn cardinal_word_ambiguity_gap(analyses: &[CardinalWordAnalysis]) -> GapOccurrence {
    let values = analyses
        .iter()
        .map(|analysis| analysis.value.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    GapOccurrence {
        kind: GapKind::AmbiguityOrSpellingVariant,
        secondary_reasons: Vec::new(),
        detail: format!("the fused cardinal surface has multiple numeric values: {values}"),
        candidate_lexeme_ids: Vec::new(),
        requested_morphological_system: Some("compound-cardinal-word".into()),
        missing_metadata: Vec::new(),
        resolver_trace: analyses
            .first()
            .map_or_else(RuleTrace::default, |analysis| analysis.rule_trace.clone()),
        suggested_action:
            "preserve every grammar-licensed numeric analysis until syntax selects one value".into(),
    }
}

pub(crate) fn analysis_ids(analyses: &[Analysis]) -> Vec<LexemeId> {
    analyses
        .iter()
        .map(|analysis| analysis.lexeme.id().clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn update_text_summary(summary: &mut TextSummary, analysis: &TextTokenAnalysis) {
    *summary
        .by_status
        .entry(status_label(analysis.status).into())
        .or_default() += 1;
    if analysis.status == TokenStatus::CyrillicNumeral {
        summary.numerals += 1;
    }
    if is_top_1_analyzed(analysis) {
        summary.top_1_analyzed += 1;
    }
    if is_top_k_analyzed(analysis) {
        summary.top_k_analyzed += 1;
    }
    if analysis.status == TokenStatus::Ambiguous {
        summary.ambiguous_tokens += 1;
    }
    if let Some(gap) = &analysis.gap {
        *summary.by_gap.entry(gap.kind.label().into()).or_default() += 1;
        if gap.kind != GapKind::AmbiguityOrSpellingVariant {
            summary.unresolved_tokens += 1;
        }
    }
}

/// Classifies one covered token by the substance of its analyses.
///
/// Numerals and fused cardinal words carry no lexeme but do resolve to a
/// unique typed value, so they count as morphologically typed and
/// lemma-unique. Uncovered tokens are ignored entirely: this measures the
/// composition of coverage, not of the corpus.
pub(crate) fn update_integrity(integrity: &mut CoverageIntegrity, analysis: &TextTokenAnalysis) {
    if !is_top_k_analyzed(analysis) {
        return;
    }
    let lexemes = analysis
        .analyses
        .iter()
        .map(|candidate| candidate.lexeme.id())
        .collect::<BTreeSet<_>>();
    if lexemes.len() > 1 {
        integrity.cross_lexeme_ambiguous += 1;
    } else {
        integrity.lemma_unique_analyzed += 1;
        if analysis.analyses.len() > 1 {
            integrity.within_lexeme_ambiguous += 1;
        }
    }
    let morphology_free = !analysis.analyses.is_empty()
        && analysis
            .analyses
            .iter()
            .all(|candidate| candidate.cell == Some(GrammarCell::LexicalForm));
    if morphology_free {
        integrity.morphology_free_analyzed += 1;
    } else {
        integrity.morphologically_typed_analyzed += 1;
    }
}

pub(crate) fn update_slice(slice: &mut CoverageSlice, analysis: &TextTokenAnalysis) {
    slice.total_tokens += 1;
    if is_top_1_analyzed(analysis) {
        slice.top_1_analyzed += 1;
    }
    if is_top_k_analyzed(analysis) {
        slice.top_k_analyzed += 1;
    }
    if analysis.status == TokenStatus::Ambiguous {
        slice.ambiguous += 1;
    }
    if analysis
        .gap
        .as_ref()
        .is_some_and(|gap| gap.kind != GapKind::AmbiguityOrSpellingVariant)
    {
        slice.unresolved += 1;
    }
}

/// The system label of a predicted cell, for the diagnostic slice only.
pub(crate) fn prediction_system(cell: GrammarCell) -> String {
    morphological_system(cell)
}

pub(crate) const fn confidence_bucket(confidence_bp: u16) -> &'static str {
    match confidence_bp {
        0..=2399 => "0-2399",
        2400..=2999 => "2400-2999",
        3000..=3399 => "3000-3399",
        _ => "3400+",
    }
}

pub(crate) fn is_top_k_analyzed(analysis: &TextTokenAnalysis) -> bool {
    let lexical = !analysis.analyses.is_empty()
        && (analysis.gap.is_none() || analysis.status == TokenStatus::Ambiguous);
    let numeric = analysis.status == TokenStatus::CyrillicNumeral
        && analysis.numeral.is_some()
        && analysis.gap.is_none();
    let cardinal_word = !analysis.cardinal_words.is_empty()
        && (analysis.gap.is_none() || analysis.status == TokenStatus::Ambiguous);
    lexical || numeric || cardinal_word
}

pub(crate) fn is_top_1_analyzed(analysis: &TextTokenAnalysis) -> bool {
    analysis.gap.is_none()
        && (analysis.analyses.len() == 1
            || (analysis.status == TokenStatus::CyrillicNumeral
                && analysis.numeral.is_some()
                && analysis.analyses.is_empty())
            || (analysis.analyses.is_empty()
                && analysis.cardinal_words.len() == 1
                && analysis.numeral.is_none()))
}

pub(crate) fn morphological_system(cell: GrammarCell) -> String {
    match cell {
        GrammarCell::LexicalForm => "lexical-form",
        GrammarCell::Indeclinable => "indeclinable",
        GrammarCell::Noun(_) => "noun",
        GrammarCell::Adjective(_) => "adjective",
        GrammarCell::FiniteVerb(cell) => match cell.tense {
            synodal_church_slavonic::FiniteTense::Present => "present",
            synodal_church_slavonic::FiniteTense::Future => "future",
            synodal_church_slavonic::FiniteTense::Past => "past",
            synodal_church_slavonic::FiniteTense::Imperfect => "imperfect",
            synodal_church_slavonic::FiniteTense::Aorist => "aorist",
        },
        GrammarCell::Imperative(_) => "imperative",
        GrammarCell::Infinitive => "infinitive",
        GrammarCell::Supine => "supine",
        GrammarCell::LParticiple(_) => "l-participle",
        GrammarCell::Participle(cell) => match (cell.tense, cell.voice) {
            (
                synodal_church_slavonic::ParticipleTense::Present,
                synodal_church_slavonic::ParticipleVoice::Active,
            ) => "present-active-participle",
            (
                synodal_church_slavonic::ParticipleTense::Present,
                synodal_church_slavonic::ParticipleVoice::Passive,
            ) => "present-passive-participle",
            (
                synodal_church_slavonic::ParticipleTense::Past,
                synodal_church_slavonic::ParticipleVoice::Active,
            ) => "past-active-participle",
            (
                synodal_church_slavonic::ParticipleTense::Past,
                synodal_church_slavonic::ParticipleVoice::Passive,
            ) => "past-passive-participle",
        },
        GrammarCell::VerbalNoun(_) => "verbal-noun",
        GrammarCell::Pronoun(_) => "pronoun",
        GrammarCell::Determiner(_) => "determiner",
        GrammarCell::Numeral(_) => "numeral",
    }
    .into()
}

pub(crate) fn spelling_key(value: &str) -> String {
    normalize_lookup_accentless(value)
        .chars()
        .map(|character| match character {
            'є' => 'е',
            'ѡ' | 'ѻ' | 'ѽ' => 'о',
            'і' | 'ї' | 'ѵ' => 'и',
            'ꙋ' | 'ᲂ' | 'ѹ' => 'у',
            'ꙗ' => 'ѧ',
            'ѣ' => 'е',
            'ѳ' => 'ф',
            value => value,
        })
        .collect()
}

pub(crate) fn contains_cyrillic(value: &str) -> bool {
    value.chars().any(is_cyrillic)
}

pub(crate) fn contains_non_cyrillic_alphabetic(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_alphabetic() && !is_cyrillic(character))
}

pub(crate) fn is_cyrillic(character: char) -> bool {
    matches!(
        character as u32,
        0x0400..=0x052f
            | 0x1c80..=0x1c8f
            | 0x2de0..=0x2dff
            | 0xa640..=0xa69f
            | 0x1e030..=0x1e08f
    )
}

pub(crate) fn escape_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

pub(crate) fn tsv_field(value: &str) -> String {
    value
        .replace('\t', " ")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_owned()
}
