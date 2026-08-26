#[allow(unused_imports)]
use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchMatchKind {
    ExactLemma,
    LemmaSubstring,
    ExactGloss,
    GlossPhrase,
    GlossWords,
    FuzzyLemma,
    FuzzyGloss,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SearchOptions {
    pub part_of_speech: Option<PartOfSpeech>,
    pub limit: usize,
    pub fuzzy: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            part_of_speech: None,
            limit: 20,
            fuzzy: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SearchMatch {
    pub entry: Entry,
    pub score: u16,
    pub matched_on: SearchMatchKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticAlignmentDecision {
    pub mapping_id: String,
    pub source_sense_id: String,
    pub target_sense_id: String,
    pub status: String,
    pub evidence_id: String,
    pub review_note: String,
}

#[must_use]
pub fn semantic_alignments() -> Vec<SemanticAlignmentDecision> {
    SEMANTIC_ALIGNMENTS
        .iter()
        .map(|row| SemanticAlignmentDecision {
            mapping_id: row.0[0].into(),
            source_sense_id: row.0[1].into(),
            target_sense_id: row.0[2].into(),
            status: row.0[3].into(),
            evidence_id: row.0[4].into(),
            review_note: row.0[5].into(),
        })
        .collect()
}

pub fn lookup(lemma: &str) -> Result<Entry> {
    let summary = morphology::lookup(lemma)?;
    entry_for(summary)
}

pub fn lookup_by_id(id: &LexemeId) -> Result<Entry> {
    let summary = morphology::advanced::lookup_by_id(id)?;
    entry_for(summary)
}

/// Returns every target entry matching a normalized lemma. Unlike `lookup`,
/// this operation preserves homographs instead of turning them into an error.
pub fn lookup_all(lemma: &str) -> Result<Vec<Entry>> {
    let parsed = SynodalWord::parse(lemma)?;
    let key = normalize_lookup_accentless(parsed.canonical());
    lexemes()?
        .into_iter()
        .filter(|lexeme| normalize_lookup_accentless(lexeme.lemma()) == key)
        .map(entry_for)
        .collect()
}

pub fn entries() -> Result<Vec<Entry>> {
    lexemes()?.into_iter().map(entry_for).collect()
}

pub fn search(query: &str, options: &SearchOptions) -> Result<Vec<SearchMatch>> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Err(Error::EmptyInput);
    }
    let query_words: BTreeSet<&str> = query.split_whitespace().collect();
    let mut matches = Vec::new();
    for entry in entries()? {
        if options
            .part_of_speech
            .is_some_and(|part_of_speech| entry.lexeme.part_of_speech() != part_of_speech)
        {
            continue;
        }
        let lemma = entry.lexeme.lemma().to_lowercase();
        let mut best = if lemma == query {
            Some((10_000, SearchMatchKind::ExactLemma))
        } else if lemma.contains(&query) {
            Some((8_800, SearchMatchKind::LemmaSubstring))
        } else {
            None
        };
        for sense in &entry.senses {
            let gloss = sense.gloss.to_lowercase();
            let gloss_words: BTreeSet<&str> = gloss
                .split(|character: char| !character.is_alphanumeric())
                .filter(|word| !word.is_empty())
                .collect();
            let candidate = if gloss == query {
                Some((9_800, SearchMatchKind::ExactGloss))
            } else if gloss.contains(&query) {
                Some((9_000, SearchMatchKind::GlossPhrase))
            } else if !query_words.is_empty() && query_words.is_subset(&gloss_words) {
                Some((8_200, SearchMatchKind::GlossWords))
            } else if options.fuzzy {
                fuzzy_score(&query, &gloss).map(|score| (score, SearchMatchKind::FuzzyGloss))
            } else {
                None
            };
            if candidate.is_some_and(|candidate| best.is_none_or(|best| candidate.0 > best.0)) {
                best = candidate;
            }
        }
        if best.is_none() && options.fuzzy {
            best = fuzzy_score(&query, &lemma).map(|score| (score, SearchMatchKind::FuzzyLemma));
        }
        if let Some((score, matched_on)) = best {
            matches.push(SearchMatch {
                entry,
                score,
                matched_on,
            });
        }
    }
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.matched_on.cmp(&right.matched_on))
            .then_with(|| left.entry.lexeme.id().cmp(right.entry.lexeme.id()))
    });
    matches.truncate(options.limit);
    Ok(matches)
}

pub fn search_gloss(query: &str) -> Result<Vec<Entry>> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Err(Error::EmptyInput);
    }
    let matching_ids: BTreeSet<LexemeId> = SENSES
        .iter()
        .filter(|sense| {
            sense.0[2].to_lowercase().contains(&query)
                || sense.0[3]
                    .split(',')
                    .any(|domain| domain.to_lowercase().contains(&query))
        })
        .map(|sense| LexemeId::from(sense.0[0]))
        .collect();
    matching_ids.iter().map(lookup_by_id).collect()
}

pub fn concordance(id: &LexemeId) -> Vec<SourceExample> {
    EXAMPLES
        .iter()
        .filter(|example| example.0[1] == id.as_str())
        .map(source_example)
        .collect()
}
