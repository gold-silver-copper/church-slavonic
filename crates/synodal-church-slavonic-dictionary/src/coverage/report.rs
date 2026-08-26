#[allow(unused_imports)]
use super::*;

/// Resolver statuses that reach a held-out type by rule rather than by a row
/// naming the type. These are the generalising statuses; everything the
/// coverage contract calls "learned" must land here.
pub const GENERALISING_STATUSES: [&str; 3] = [
    "synodal-normative-table",
    "synodal-productive-rule",
    "synodal-irregular-override",
];

/// The resolver status that reaches a held-out type by an exact row citing
/// the type itself: memorisation, grandfathered and capped.
pub const MEMORISING_STATUS: &str = "exact-synodal-attestation";

impl CoverageReport {
    /// Held-out tokens reached by normative table, productive rule, or
    /// irregular override. This is the headline measure of the program.
    #[must_use]
    pub fn held_out_generalised(&self) -> usize {
        GENERALISING_STATUSES
            .iter()
            .map(|status| {
                self.held_out_type_status
                    .get(*status)
                    .copied()
                    .unwrap_or_default()
            })
            .sum()
    }

    /// Held-out tokens reached by an exact row citing the held-out type.
    #[must_use]
    pub fn held_out_memorised(&self) -> usize {
        self.held_out_type_status
            .get(MEMORISING_STATUS)
            .copied()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn markdown(&self) -> String {
        let basis_points = |value: usize, total: usize| {
            value
                .saturating_mul(10_000)
                .checked_div(total)
                .unwrap_or_default()
        };
        let mut output = String::from("# Synodal corpus coverage\n");
        if self.held_out_type_coverage.total_tokens > 0 {
            let held = &self.held_out_type_coverage;
            let generalised = self.held_out_generalised();
            let memorised = self.held_out_memorised();
            let ambiguous = self
                .held_out_type_status
                .get("ambiguous")
                .copied()
                .unwrap_or_default();
            output.push_str(&format!(
                "\n## Type-disjoint holdout\n\nThis is the headline measure. The corpus partition split is passage-disjoint,\nso an exact row sourced from a `source` passage closes its own held-out twin.\nThis slice holds out normalized *types* instead, selected by a content hash that\ncannot be tuned, and is the only measurement here that shows generalisation to\nsurfaces the reviewed data has never seen. Coverage that arrives as\n`exact-synodal-attestation` is a row citing the held-out type itself and is\nmemorisation; `synodal-normative-table`, `synodal-productive-rule` and\n`synodal-irregular-override` coverage is generalisation. Corpus-wide top-k\nrising while `generalised` stays flat is memorising.\n\n- Held-out types present: {}\n- Held-out tokens: {}\n\n| Outcome | Tokens | Share of held-out |\n|---|---:|---:|\n| **generalised** (by rule) | {generalised} | {} bp |\n| memorised (exact row) | {memorised} | {} bp |\n| ambiguous | {ambiguous} | {} bp |\n| unresolved | {} | {} bp |\n| top-k (any analysis) | {} | {} bp |\n| top-1 | {} | {} bp |\n\n### Held-out tokens by resolver status\n\n| Resolver status | Tokens | Share of held-out |\n|---|---:|---:|\n",
                self.held_out_types,
                held.total_tokens,
                basis_points(generalised, held.total_tokens),
                basis_points(memorised, held.total_tokens),
                basis_points(ambiguous, held.total_tokens),
                held.unresolved,
                basis_points(held.unresolved, held.total_tokens),
                held.top_k_analyzed,
                basis_points(held.top_k_analyzed, held.total_tokens),
                held.top_1_analyzed,
                basis_points(held.top_1_analyzed, held.total_tokens),
            ));
            for (status, tokens) in &self.held_out_type_status {
                output.push_str(&format!(
                    "| `{status}` | {tokens} | {} bp |\n",
                    basis_points(*tokens, held.total_tokens)
                ));
            }
            output.push_str("\n### Held-out tokens by morphological system\n\nA wave aimed at one system must be visible landing in that system.\n\n| System | Held-out | Generalised | Memorised | Unresolved |\n|---|---:|---:|---:|---:|\n");
            for (system, statuses) in &self.held_out_type_status_by_system {
                let count = |label: &str| statuses.get(label).copied().unwrap_or_default();
                let total: usize = statuses.values().sum();
                let generalised: usize = GENERALISING_STATUSES.iter().map(|s| count(s)).sum();
                output.push_str(&format!(
                    "| `{system}` | {total} | {generalised} | {} | {} |\n",
                    count(MEMORISING_STATUS),
                    count("unresolved"),
                ));
            }
        }
        output.push_str(&format!(
            "\n## Corpus-wide coverage\n\n- Passages: {}\n- Tokens: {}\n- Types: {}\n- Top-1 analyzed: {} ({} bp)\n- Top-k analyzed: {} ({} bp)\n- Ambiguous: {}\n- Unresolved: {}\n\n## Gap categories\n\n| Category | Tokens |\n|---|---:|\n",
            self.passages,
            self.summary.total_tokens,
            self.token_types,
            self.summary.top_1_analyzed,
            basis_points(self.summary.top_1_analyzed, self.summary.total_tokens),
            self.summary.top_k_analyzed,
            basis_points(self.summary.top_k_analyzed, self.summary.total_tokens),
            self.summary.ambiguous,
            self.summary.unresolved,
        ));
        for kind in GapKind::ALL {
            output.push_str(&format!(
                "| `{}` | {} |\n",
                kind.label(),
                self.by_gap.get(kind.label()).copied().unwrap_or_default()
            ));
        }
        output.push_str(
            "\n## Coverage composition\n\nStrict top-k counts tokens that have *any* analysis. These measures describe what\nthat coverage is made of, so recall cannot be bought with rows that commit to no\nmorphology, and so a fall in unique-reading counts can be attributed rather than\nassumed. `morphology-free` tokens carry only `lexical-form` readings.\n`lemma-unique` is not capped by syncretism the way top-1 is.\n\n| Measure | Tokens | Share of top-k |\n|---|---:|---:|\n",
        );
        for (label, value) in [
            (
                "morphologically typed",
                self.integrity.morphologically_typed_analyzed,
            ),
            ("morphology-free", self.integrity.morphology_free_analyzed),
            ("lemma-unique", self.integrity.lemma_unique_analyzed),
            (
                "within-lexeme ambiguous (syncretism)",
                self.integrity.within_lexeme_ambiguous,
            ),
            (
                "cross-lexeme ambiguous (homonymy)",
                self.integrity.cross_lexeme_ambiguous,
            ),
        ] {
            output.push_str(&format!(
                "| {label} | {value} | {} bp |\n",
                basis_points(value, self.summary.top_k_analyzed)
            ));
        }
        output.push_str("\n## Estimated recovery routes\n\nThese are diagnostic estimates, not admitted lexical identities or guaranteed recoveries.\n\n| Route | Tokens |\n|---|---:|\n");
        for route in [
            RecoveryRoute::ExactEvidence,
            RecoveryRoute::ReviewedClass,
            RecoveryRoute::ReviewedPrincipalPart,
            RecoveryRoute::AbbreviationRegistry,
            RecoveryRoute::SpellingVariant,
            RecoveryRoute::UnsupportedFormation,
            RecoveryRoute::UngroupedUnknown,
        ] {
            output.push_str(&format!(
                "| `{}` | {} |\n",
                route.label(),
                self.estimated_recovery_by_route
                    .get(route.label())
                    .copied()
                    .unwrap_or_default(),
            ));
        }
        if !self.predicted_unresolved_by_system.is_empty() {
            output.push_str("\n## Exploratory predictions over the unresolved remainder\n\nDiagnostic only. These tokens have no reviewed reading; the corpus-free\nsegmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`, reachable only under\n`GenerationPolicy::Exploratory`) can offer a typed hypothesis for them. They\nnever count toward strict top-k and no sealed floor reads this table; the\nmasked precision gate lives in `reports/synodal-prediction-precision.md`.\n\n| Top prediction's system | Tokens |\n|---|---:|\n");
            for (system, tokens) in &self.predicted_unresolved_by_system {
                output.push_str(&format!("| `{system}` | {tokens} |\n"));
            }
            output.push_str("\n| Confidence bucket (bp) | Tokens |\n|---|---:|\n");
            for (bucket, tokens) in &self.predicted_unresolved_by_confidence {
                output.push_str(&format!("| {bucket} | {tokens} |\n"));
            }
        }
        output.push_str("\n## Unresolved tokens by probable family\n\n| Family diagnostic | Tokens | Documents | Route | Surfaces |\n|---|---:|---:|---|---|\n");
        let mut diagnostics: Vec<_> = self.unresolved_by_probable_family.values().collect();
        diagnostics.retain(|diagnostic| diagnostic.top_k_uncovered_token_frequency > 0);
        diagnostics.sort_by(|left, right| {
            right
                .top_k_uncovered_token_frequency
                .cmp(&left.top_k_uncovered_token_frequency)
                .then_with(|| left.probable_family_id.cmp(&right.probable_family_id))
        });
        for diagnostic in diagnostics.into_iter().take(100) {
            output.push_str(&format!(
                "| `{}` | {} | {} | `{}` | {} |\n",
                escape_markdown(&diagnostic.probable_family_id),
                diagnostic.top_k_uncovered_token_frequency,
                diagnostic.document_frequency,
                diagnostic.recovery_route.label(),
                escape_markdown(&diagnostic.surfaces.join(", ")),
            ));
        }
        output.push_str("\n## Coverage by corpus\n\n| Corpus | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |\n|---|---:|---:|---:|---:|---:|\n");
        for (corpus, slice) in &self.by_corpus {
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                escape_markdown(corpus),
                slice.total_tokens,
                slice.top_1_analyzed,
                slice.top_k_analyzed,
                slice.ambiguous,
                slice.unresolved,
            ));
        }
        output.push_str(
            "\n## Coverage by source\n\n| Source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |\n|---|---:|---:|---:|---:|---:|\n",
        );
        for (source, slice) in &self.by_source {
            output.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                escape_markdown(source),
                slice.total_tokens,
                slice.top_1_analyzed,
                slice.top_k_analyzed,
                slice.ambiguous,
                slice.unresolved,
            ));
        }
        output.push_str(
            "\n## Coverage by partition\n\n| Partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |\n|---|---:|---:|---:|---:|---:|\n",
        );
        for (partition, slice) in &self.by_partition {
            output.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                escape_markdown(partition),
                slice.total_tokens,
                slice.top_1_analyzed,
                slice.top_k_analyzed,
                slice.ambiguous,
                slice.unresolved,
            ));
        }
        output.push_str(
            "\n## Coverage by source and partition\n\n| Source/partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |\n|---|---:|---:|---:|---:|---:|\n",
        );
        for (source_partition, slice) in &self.by_source_partition {
            output.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                escape_markdown(source_partition),
                slice.total_tokens,
                slice.top_1_analyzed,
                slice.top_k_analyzed,
                slice.ambiguous,
                slice.unresolved,
            ));
        }
        output.push_str(
            "\n## Gap categories by source\n\n| Source | Category | Tokens |\n|---|---|---:|\n",
        );
        for (source, gaps) in &self.by_source_gap {
            for kind in GapKind::ALL {
                let count = gaps.get(kind.label()).copied().unwrap_or_default();
                if count > 0 {
                    output.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        escape_markdown(source),
                        kind.label(),
                        count,
                    ));
                }
            }
        }
        output.push_str(
            "\n## Gap categories by partition\n\n| Partition | Category | Tokens |\n|---|---|---:|\n",
        );
        for (partition, gaps) in &self.by_partition_gap {
            for kind in GapKind::ALL {
                let count = gaps.get(kind.label()).copied().unwrap_or_default();
                if count > 0 {
                    output.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        escape_markdown(partition),
                        kind.label(),
                        count,
                    ));
                }
            }
        }
        output.push_str(
            "\n## Review queue\n\n| Rank | Gap | Token | Frequency | Documents | Action |\n|---:|---|---|---:|---:|---|\n",
        );
        for item in &self.review_queue {
            output.push_str(&format!(
                "| {} | `{}` | `{}` | {} | {} | {} |\n",
                item.rank,
                item.kind.label(),
                escape_markdown(&item.sample),
                item.frequency,
                item.document_frequency,
                escape_markdown(&item.suggested_action),
            ));
        }
        output
    }

    #[must_use]
    pub fn gaps_tsv(&self) -> String {
        let mut output = String::from(
            "rank\tkind\tnormalized\tsample\tfrequency\tdocument_frequency\tcandidate_lexeme_ids\tsuggested_action\n",
        );
        for item in &self.review_queue {
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                item.rank,
                item.kind.label(),
                tsv_field(&item.normalized),
                tsv_field(&item.sample),
                item.frequency,
                item.document_frequency,
                item.candidate_lexeme_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                tsv_field(&item.suggested_action),
            ));
        }
        output
    }

    /// Renders every strict top-k-uncovered surface/status row. This is the
    /// authoritative work frontier; unlike [`Self::gaps_tsv`], it is not
    /// truncated and also accounts for any uncovered status without a gap.
    #[must_use]
    pub fn uncovered_frontier_tsv(&self) -> String {
        let mut output = String::from(
            "rank\tstatus\tkind\tnormalized\tsample\ttoken_frequency\tdocument_frequency\tcorpora\tsource_ids\tpartitions\tcandidate_lexeme_ids\trequested_morphological_system\tmissing_metadata\tcontexts\tsuggested_action\n",
        );
        for (index, item) in self.uncovered_frontier.iter().enumerate() {
            let contexts = item
                .contexts
                .iter()
                .map(|context| {
                    format!(
                        "{}@{}:{}:{} {}",
                        context.document,
                        context.passage,
                        context.line,
                        context.column,
                        context.excerpt,
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                index + 1,
                status_label(item.status),
                item.kind.map_or("", GapKind::label),
                tsv_field(&item.normalized),
                tsv_field(&item.sample),
                item.token_frequency,
                item.document_frequency,
                item.corpora.join(","),
                item.source_ids.join(","),
                item.partitions.join(","),
                item.candidate_lexeme_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                item.requested_morphological_system.as_deref().unwrap_or(""),
                item.missing_metadata
                    .iter()
                    .map(|field| format!("{field:?}"))
                    .collect::<Vec<_>>()
                    .join(","),
                tsv_field(&contexts),
                tsv_field(&item.suggested_action),
            ));
        }
        output
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProbableFamilyAggregate {
    id: String,
    frequency: usize,
    top_k_uncovered_frequency: usize,
    documents: BTreeSet<String>,
    surfaces: BTreeSet<String>,
    candidate_lexeme_ids: BTreeSet<LexemeId>,
    recovery_route: RecoveryRoute,
    assumption: String,
}

impl ProbableFamilyAggregate {
    pub(crate) fn new(id: String, recovery_route: RecoveryRoute, assumption: String) -> Self {
        Self {
            id,
            frequency: 0,
            top_k_uncovered_frequency: 0,
            documents: BTreeSet::new(),
            surfaces: BTreeSet::new(),
            candidate_lexeme_ids: BTreeSet::new(),
            recovery_route,
            assumption,
        }
    }

    pub(crate) fn observe(&mut self, analysis: &TextTokenAnalysis, gap: &GapOccurrence, document: &str) {
        self.frequency += 1;
        if !is_top_k_analyzed(analysis) {
            self.top_k_uncovered_frequency += 1;
            self.documents.insert(document.into());
        }
        self.surfaces.insert(analysis.token.original.clone());
        self.candidate_lexeme_ids
            .extend(gap.candidate_lexeme_ids.iter().cloned());
    }

    pub(crate) fn finish(self) -> ProbableFamilyDiagnostic {
        ProbableFamilyDiagnostic {
            probable_family_id: self.id,
            token_frequency: self.frequency,
            top_k_uncovered_token_frequency: self.top_k_uncovered_frequency,
            document_frequency: self.documents.len(),
            surfaces: self.surfaces.into_iter().collect(),
            candidate_lexeme_ids: self.candidate_lexeme_ids.into_iter().collect(),
            recovery_route: self.recovery_route,
            assumption: self.assumption,
        }
    }
}

pub(crate) fn probable_family(
    analysis: &TextTokenAnalysis,
    gap: &GapOccurrence,
) -> (String, RecoveryRoute, String) {
    let route = match gap.kind {
        GapKind::MissingDeclensionOrClass => RecoveryRoute::ReviewedClass,
        GapKind::MissingVerbPrincipalPart => RecoveryRoute::ReviewedPrincipalPart,
        GapKind::UnsupportedFormation if !gap.candidate_lexeme_ids.is_empty() => {
            RecoveryRoute::UnsupportedFormation
        }
        GapKind::MissingAccentOrOrthographicMetadata | GapKind::AmbiguityOrSpellingVariant => {
            RecoveryRoute::SpellingVariant
        }
        GapKind::UnknownLexeme if has_abbreviation_marks(&analysis.token.original) => {
            RecoveryRoute::AbbreviationRegistry
        }
        GapKind::UnknownLexeme => RecoveryRoute::UngroupedUnknown,
        GapKind::UnsupportedFormation => RecoveryRoute::ExactEvidence,
    };
    if gap.candidate_lexeme_ids.len() == 1 {
        return (
            FamilyId::for_lexeme(&gap.candidate_lexeme_ids[0]).to_string(),
            route,
            "one reviewed lexeme is compatible with this diagnostic; the requested cell still requires review".into(),
        );
    }
    if gap.candidate_lexeme_ids.len() > 1 {
        return (
            format!(
                "ambiguous-family:{}",
                gap.candidate_lexeme_ids
                    .iter()
                    .map(LexemeId::as_str)
                    .collect::<Vec<_>>()
                    .join("+")
            ),
            route,
            "multiple reviewed lexemes remain compatible; no family was selected".into(),
        );
    }

    let normalized = analysis.token.normalized.as_str();
    let recognized = if normalized.starts_with("реч") || normalized.starts_with("рц") {
        Some((
            "diagnostic-family:рещи",
            "shared surface material suggests the reviewed рещи family; stem similarity alone does not prove identity",
        ))
    } else if matches!(
        normalized,
        "весь"
            | "всѧ"
            | "все"
            | "вси"
            | "всѣхъ"
            | "всѣмъ"
            | "всѣми"
            | "всю"
            | "всему"
            | "всей"
            | "всеѧ"
            | "всего"
            | "всею"
    ) {
        Some((
            "diagnostic-family:весь",
            "shared вс- material suggests весь; pronominal and unrelated identities must remain possible",
        ))
    } else if normalized.starts_with("сын") {
        Some((
            "diagnostic-family:сынъ",
            "shared сын- material suggests сынъ; the consonantal plural alternant still requires evidence",
        ))
    } else if normalized.starts_with("земл") || normalized.starts_with("земе") {
        Some((
            "diagnostic-family:землѧ",
            "shared земл-/земе- material suggests землѧ; the alternation and cell remain unproved",
        ))
    } else if normalized.starts_with("господ")
        || (normalized.starts_with("гд") && has_abbreviation_marks(&analysis.token.original))
    {
        Some((
            "diagnostic-family:господь",
            "expanded or contracted surface suggests господь; titlo scope and grammatical cell remain review requirements",
        ))
    } else if normalized.starts_with("ꙗкож") {
        Some((
            "diagnostic-family:ꙗкоже",
            "surface similarity groups the token for review while preserving adverb/conjunction ambiguity",
        ))
    } else if normalized == "ꙗкѡ" || normalized == "яко" {
        Some((
            "diagnostic-family:ꙗкѡ",
            "surface identity groups the token while preserving all reviewed syntactic identities",
        ))
    } else {
        None
    };
    if let Some((id, assumption)) = recognized {
        return (
            id.into(),
            if route == RecoveryRoute::UngroupedUnknown {
                RecoveryRoute::ExactEvidence
            } else {
                route
            },
            assumption.into(),
        );
    }
    (
        format!("ungrouped:{}", analysis.token.normalized),
        route,
        "no reviewed lexical identity or conservative high-impact family diagnostic is available"
            .into(),
    )
}

pub(crate) fn has_abbreviation_marks(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(character, '\u{0483}' | '\u{0487}')
            || ('\u{2de0}'..='\u{2dff}').contains(&character)
    })
}

#[derive(Clone, Debug)]
pub(crate) struct CoverageFrontierAggregate {
    item: CoverageFrontierItem,
    documents: BTreeSet<String>,
    corpora: BTreeSet<String>,
    source_ids: BTreeSet<String>,
    partitions: BTreeSet<String>,
    contexts: BTreeSet<(String, String, usize, usize, String)>,
}

impl CoverageFrontierAggregate {
    pub(crate) fn new(analysis: &TextTokenAnalysis) -> Self {
        let gap = analysis.gap.as_ref();
        Self {
            item: CoverageFrontierItem {
                status: analysis.status,
                kind: gap.map(|gap| gap.kind),
                normalized: analysis.token.normalized.clone(),
                sample: analysis.token.original.clone(),
                token_frequency: 0,
                document_frequency: 0,
                corpora: Vec::new(),
                source_ids: Vec::new(),
                partitions: Vec::new(),
                candidate_lexeme_ids: gap
                    .map(|gap| gap.candidate_lexeme_ids.clone())
                    .unwrap_or_default(),
                requested_morphological_system: gap
                    .and_then(|gap| gap.requested_morphological_system.clone()),
                missing_metadata: gap
                    .map(|gap| gap.missing_metadata.clone())
                    .unwrap_or_default(),
                suggested_action: gap.map_or_else(
                    || "review why this status remains outside strict top-k".into(),
                    |gap| gap.suggested_action.clone(),
                ),
                contexts: Vec::new(),
            },
            documents: BTreeSet::new(),
            corpora: BTreeSet::new(),
            source_ids: BTreeSet::new(),
            partitions: BTreeSet::new(),
            contexts: BTreeSet::new(),
        }
    }

    pub(crate) fn observe(&mut self, passage: &CoveragePassage, document: &str, analysis: &TextTokenAnalysis) {
        self.item.token_frequency += 1;
        self.documents.insert(document.into());
        self.corpora.insert(passage.corpus.clone());
        self.source_ids.insert(passage.source_id.clone());
        self.partitions.insert(passage.partition.clone());
        if self.contexts.len() < 4 {
            self.contexts.insert((
                document.into(),
                passage.passage.clone(),
                analysis.token.line,
                analysis.token.column,
                context_excerpt(
                    &passage.text,
                    analysis.token.byte_start,
                    analysis.token.byte_end,
                ),
            ));
        }
        if let Some(gap) = &analysis.gap {
            self.item
                .candidate_lexeme_ids
                .extend(gap.candidate_lexeme_ids.iter().cloned());
            self.item.candidate_lexeme_ids.sort();
            self.item.candidate_lexeme_ids.dedup();
            self.item
                .missing_metadata
                .extend(gap.missing_metadata.iter().copied());
            self.item.missing_metadata.sort();
            self.item.missing_metadata.dedup();
        }
    }

    pub(crate) fn finish(mut self) -> CoverageFrontierItem {
        self.item.document_frequency = self.documents.len();
        self.item.corpora = self.corpora.into_iter().collect();
        self.item.source_ids = self.source_ids.into_iter().collect();
        self.item.partitions = self.partitions.into_iter().collect();
        self.item.contexts = self
            .contexts
            .into_iter()
            .take(2)
            .map(|(document, passage, line, column, excerpt)| GapContext {
                document,
                passage,
                line,
                column,
                excerpt,
            })
            .collect();
        self.item
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GapAggregate {
    record: GapRecord,
    documents: BTreeSet<String>,
    top_k_uncovered_documents: BTreeSet<String>,
    corpora: BTreeSet<String>,
    source_ids: BTreeSet<String>,
    editions: BTreeSet<String>,
    partitions: BTreeSet<String>,
    source_recensions: BTreeSet<String>,
    contexts: BTreeSet<(String, String, usize, usize, String)>,
}

impl GapAggregate {
    pub(crate) fn new(passage: &CoveragePassage, analysis: &TextTokenAnalysis, gap: &GapOccurrence) -> Self {
        Self {
            record: GapRecord {
                kind: gap.kind,
                original: analysis.token.original.clone(),
                normalized: analysis.token.normalized.clone(),
                corpus: passage.corpus.clone(),
                source_id: passage.source_id.clone(),
                work: passage.work.clone(),
                edition: passage.edition.clone(),
                passage: passage.passage.clone(),
                partition: passage.partition.clone(),
                source_recension: passage.source_recension.clone(),
                corpora: Vec::new(),
                source_ids: Vec::new(),
                editions: Vec::new(),
                partitions: Vec::new(),
                source_recensions: Vec::new(),
                documents: Vec::new(),
                contexts: Vec::new(),
                byte_start: analysis.token.byte_start,
                byte_end: analysis.token.byte_end,
                line: analysis.token.line,
                column: analysis.token.column,
                candidate_lexeme_ids: gap.candidate_lexeme_ids.clone(),
                requested_morphological_system: gap.requested_morphological_system.clone(),
                generation_policy: GenerationPolicy::Strict,
                orthography_profile: OrthographyProfile::Expanded,
                resolver_trace: gap.resolver_trace.clone(),
                missing_metadata: gap.missing_metadata.clone(),
                secondary_reasons: gap.secondary_reasons.clone(),
                detail: gap.detail.clone(),
                frequency: 0,
                document_frequency: 0,
                top_k_uncovered_frequency: 0,
                top_k_uncovered_documents: Vec::new(),
                suggested_action: gap.suggested_action.clone(),
            },
            documents: BTreeSet::new(),
            top_k_uncovered_documents: BTreeSet::new(),
            corpora: BTreeSet::new(),
            source_ids: BTreeSet::new(),
            editions: BTreeSet::new(),
            partitions: BTreeSet::new(),
            source_recensions: BTreeSet::new(),
            contexts: BTreeSet::new(),
        }
    }

    pub(crate) fn observe(
        &mut self,
        passage: &CoveragePassage,
        document: &str,
        analysis: &TextTokenAnalysis,
        gap: &GapOccurrence,
    ) {
        self.record.frequency += 1;
        self.documents.insert(document.into());
        if !is_top_k_analyzed(analysis) {
            self.record.top_k_uncovered_frequency += 1;
            self.top_k_uncovered_documents.insert(document.into());
        }
        self.corpora.insert(passage.corpus.clone());
        self.source_ids.insert(passage.source_id.clone());
        self.editions.insert(passage.edition.clone());
        self.partitions.insert(passage.partition.clone());
        self.source_recensions
            .insert(passage.source_recension.clone());
        if self.contexts.len() < 8 {
            self.contexts.insert((
                document.into(),
                passage.passage.clone(),
                analysis.token.line,
                analysis.token.column,
                context_excerpt(
                    &passage.text,
                    analysis.token.byte_start,
                    analysis.token.byte_end,
                ),
            ));
        }
        self.record
            .candidate_lexeme_ids
            .extend(gap.candidate_lexeme_ids.iter().cloned());
        self.record.candidate_lexeme_ids.sort();
        self.record.candidate_lexeme_ids.dedup();
        self.record
            .missing_metadata
            .extend(gap.missing_metadata.iter().copied());
        self.record.missing_metadata.sort();
        self.record.missing_metadata.dedup();
        self.record
            .secondary_reasons
            .extend(gap.secondary_reasons.iter().copied());
        self.record.secondary_reasons.sort();
        self.record.secondary_reasons.dedup();
    }

    pub(crate) fn finish(mut self, options: &CheckTextOptions) -> GapRecord {
        self.record.document_frequency = self.documents.len();
        self.record.documents = self.documents.into_iter().collect();
        self.record.top_k_uncovered_documents =
            self.top_k_uncovered_documents.into_iter().collect();
        self.record.contexts = self
            .contexts
            .into_iter()
            .take(5)
            .map(|(document, passage, line, column, excerpt)| GapContext {
                document,
                passage,
                line,
                column,
                excerpt,
            })
            .collect();
        self.record.corpora = self.corpora.into_iter().collect();
        self.record.source_ids = self.source_ids.into_iter().collect();
        self.record.editions = self.editions.into_iter().collect();
        self.record.partitions = self.partitions.into_iter().collect();
        self.record.source_recensions = self.source_recensions.into_iter().collect();
        self.record.generation_policy = options.generation_policy;
        self.record.orthography_profile = options.orthography_profile;
        self.record
    }
}

pub(crate) fn context_excerpt(text: &str, byte_start: usize, byte_end: usize) -> String {
    let start = text[..byte_start]
        .char_indices()
        .rev()
        .nth(8)
        .map_or(0, |(index, _)| index);
    let end = text[byte_end..]
        .char_indices()
        .nth(8)
        .map_or(text.len(), |(index, _)| byte_end + index);
    text[start..end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
