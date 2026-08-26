//! Rule-based adjective declension.
//!
//! Since the phase-4 adjective merge (docs/UNIFIED_LANGUAGE_PROMPT.md) the
//! shared hard/soft short and long ending tables live in the merged kernel
//! `church_slavonic_core::adjective`; this module is the family adapter that
//! keeps the public API, the citation parsing, the palatalization seams, the
//! jer-j workstems, the long-only identities, and the comparative
//! principal-part machinery.

use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, ComparativeFormation, Gender,
    InflectionError, Number, PredictedForm, RuleId, RuleStep,
};
use church_slavonic_core::{Recension, adjective as kernel};

const OCS: Recension = Recension::OldChurchSlavonic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveLexeme {
    pub lemma: String,
    pub class: AdjectiveClass,
}

/// The three ordinary class `2/a` adjectives whose dictionary citation is the
/// long masculine nominative singular because their short paradigm is absent.
///
/// Polivanova lists this inventory exhaustively in §§285 and 305. Keeping it
/// typed prevents a citation such as `прочии` from being misread as a short
/// soft adjective in `-и`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LongOnlyAdjectiveIdentity {
    InterrogativeKotoryi,
    OtherProkyi,
    OtherProchii,
}

impl LongOnlyAdjectiveIdentity {
    pub const ALL: [Self; 3] = [
        Self::InterrogativeKotoryi,
        Self::OtherProkyi,
        Self::OtherProchii,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::InterrogativeKotoryi => "которꙑи",
            Self::OtherProkyi => "прокꙑи",
            Self::OtherProchii => "прочии",
        }
    }

    pub const fn class(self) -> AdjectiveClass {
        match self {
            Self::InterrogativeKotoryi | Self::OtherProkyi => AdjectiveClass::Hard,
            Self::OtherProchii => AdjectiveClass::Soft,
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::InterrogativeKotoryi => &["которꙑи", "которыи"],
            Self::OtherProkyi => &["прокꙑи", "прокыи"],
            Self::OtherProchii => &["прочии"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    const fn stem(self) -> &'static str {
        match self {
            Self::InterrogativeKotoryi => "котор",
            Self::OtherProkyi => "прок",
            Self::OtherProchii => "проч",
        }
    }
}

/// Decline one member of the exhaustively listed `plenum tantum` inventory.
pub fn decline_long_only(
    identity: LongOnlyAdjectiveIdentity,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(identity.canonical_lemma())?;
    if cell.form != AdjectiveForm::Long {
        return Err(InflectionError::historically_invalid(
            lemma,
            crate::RequestedCell::Adjective(cell),
        ));
    }
    decline_validated_stem(identity.stem(), identity.class(), cell, &lemma)
}

/// The two principal parts needed to inflect one OCS comparative.
///
/// `syncopated_citation` is the short masculine nominative singular (`новѣи`,
/// `грѫбл҄ь`); `expanded_citation` is the short feminine nominative singular
/// (`новѣиши`, `грѫбл҄ьши`). Requiring both prevents the engine from guessing
/// the lexically restricted consonant alternations of old comparatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparativeLexeme {
    pub positive_lemma: String,
    pub syncopated_citation: String,
    pub expanded_citation: String,
    pub formation: ComparativeFormation,
}

/// Build the productive new comparative from an explicitly classified positive
/// adjective. Final velars undergo first palatalization and select surface
/// `-аи-`; all other bases select `-ѣи-`.
pub fn productive_new_comparative(
    positive: &AdjectiveLexeme,
) -> Result<ComparativeLexeme, InflectionError> {
    let lemma = crate::orthography::canonical_display(&positive.lemma)?;
    if LongOnlyAdjectiveIdentity::classify_source_union_lemma(&lemma).is_some() {
        return Err(InflectionError::InvalidInput {
            reason: "a productive comparative requires a short positive adjective citation"
                .to_string(),
        });
    }
    let (stem, long_only) = adjective_citation_stem(&lemma, positive.class)?;
    if long_only {
        return Err(InflectionError::InvalidInput {
            reason: "a productive comparative requires a short positive adjective citation"
                .to_string(),
        });
    }
    let (base, suffix) = if stem.ends_with(['к', 'г', 'х']) {
        (palatalize(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")]), "аи")
    } else {
        (stem.to_string(), "ѣи")
    };
    let syncopated_citation = format!("{base}{suffix}");
    Ok(ComparativeLexeme {
        positive_lemma: lemma,
        expanded_citation: format!("{syncopated_citation}ши"),
        syncopated_citation,
        formation: ComparativeFormation::New,
    })
}

/// Inflect a comparative from its independently supplied principal parts.
///
/// Comparatives use a syncopated stem in precisely three source-described
/// direct cells and an expanded soft-adjective stem elsewhere. Four expanded
/// cells take the alien endings `-и/-иꙗ` and `-е/-еи`.
pub fn decline_comparative(
    lexeme: &ComparativeLexeme,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let lexeme = validate_comparative(lexeme)?;
    let rule_id = match lexeme.formation {
        ComparativeFormation::New => RuleId::AdjectiveComparativeNew,
        ComparativeFormation::Old => RuleId::AdjectiveComparativeOld,
    };

    let text = if is_syncopated_comparative_cell(cell) {
        syncopated_comparative_form(&lexeme, cell)?
    } else {
        let expanded_stem = lexeme
            .expanded_citation
            .strip_suffix('и')
            .ok_or_else(|| contradictory_comparative(&lexeme))?;
        if let Some(ending) = comparative_alien_ending(cell) {
            format!("{expanded_stem}{ending}")
        } else {
            decline_validated_stem(
                expanded_stem,
                AdjectiveClass::Soft,
                cell,
                &lexeme.expanded_citation,
            )?
            .text
        }
    };

    Ok(PredictedForm {
        text: text.clone(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: lexeme.positive_lemma,
            after: text,
            reason: "select the comparative principal-part stem and attach its agreement ending",
        }],
    })
}

/// Form and decline the source-described absolute superlative in `прѣ-`.
///
/// Prefixation leaves the positive adjective's lexical declension class intact.
pub fn decline_pre_superlative(
    positive: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(&positive.lemma)?;
    let derived = AdjectiveLexeme {
        lemma: format!("прѣ{lemma}"),
        class: positive.class,
    };
    let declined = decline(&derived, cell)?;
    let rule_id = RuleId::AdjectiveSuperlativePre;
    let mut trace = Vec::with_capacity(declined.trace.len() + 1);
    trace.push(RuleStep {
        rule_id,
        before: lemma,
        after: derived.lemma,
        reason: "prefix прѣ- to form an absolute superlative adjective",
    });
    trace.extend(declined.trace);
    Ok(PredictedForm {
        text: declined.text,
        rule_id,
        trace,
    })
}

fn validate_comparative(lexeme: &ComparativeLexeme) -> Result<ComparativeLexeme, InflectionError> {
    let normalized = ComparativeLexeme {
        positive_lemma: crate::orthography::canonical_display(&lexeme.positive_lemma)?,
        syncopated_citation: crate::orthography::canonical_display(&lexeme.syncopated_citation)?,
        expanded_citation: crate::orthography::canonical_display(&lexeme.expanded_citation)?,
        formation: lexeme.formation,
    };
    let valid_syncopated_ending = match normalized.formation {
        ComparativeFormation::New => normalized.syncopated_citation.ends_with('и'),
        ComparativeFormation::Old => normalized.syncopated_citation.ends_with('ь'),
    };
    if !valid_syncopated_ending
        || normalized.expanded_citation != format!("{}ши", normalized.syncopated_citation)
    {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "the {} comparative principal parts are contradictory",
                normalized.formation.code()
            ),
        });
    }
    Ok(normalized)
}

fn is_syncopated_comparative_cell(cell: AdjectiveCell) -> bool {
    if cell.number != Number::Singular {
        return false;
    }
    matches!(
        (cell.form, cell.case, cell.gender, cell.animacy),
        (
            AdjectiveForm::Short,
            Case::Nominative,
            Gender::Masculine | Gender::Neuter,
            _
        ) | (AdjectiveForm::Short, Case::Accusative, Gender::Neuter, _)
            | (
                AdjectiveForm::Short | AdjectiveForm::Long,
                Case::Accusative,
                Gender::Masculine,
                Animacy::Inanimate,
            )
            | (AdjectiveForm::Long, Case::Nominative, Gender::Masculine, _)
    )
}

fn syncopated_comparative_form(
    lexeme: &ComparativeLexeme,
    cell: AdjectiveCell,
) -> Result<String, InflectionError> {
    let text = match (lexeme.formation, cell.form, cell.gender) {
        (_, AdjectiveForm::Short, Gender::Masculine) => lexeme.syncopated_citation.clone(),
        (ComparativeFormation::New, AdjectiveForm::Short, Gender::Neuter) => {
            let stem = lexeme
                .syncopated_citation
                .strip_suffix('и')
                .ok_or_else(|| contradictory_comparative(lexeme))?;
            format!("{stem}ѥ")
        }
        (ComparativeFormation::Old, AdjectiveForm::Short, Gender::Neuter) => {
            let stem = lexeme
                .syncopated_citation
                .strip_suffix('ь')
                .ok_or_else(|| contradictory_comparative(lexeme))?;
            format!("{stem}е")
        }
        (ComparativeFormation::New, AdjectiveForm::Long, Gender::Masculine) => {
            format!("{}и", lexeme.syncopated_citation)
        }
        (ComparativeFormation::Old, AdjectiveForm::Long, Gender::Masculine) => {
            let stem = lexeme
                .syncopated_citation
                .strip_suffix('ь')
                .ok_or_else(|| contradictory_comparative(lexeme))?;
            format!("{stem}ии")
        }
        _ => return Err(contradictory_comparative(lexeme)),
    };
    Ok(text)
}

fn contradictory_comparative(lexeme: &ComparativeLexeme) -> InflectionError {
    InflectionError::InvalidInput {
        reason: format!(
            "the {} comparative principal parts are contradictory",
            lexeme.formation.code()
        ),
    }
}

fn comparative_alien_ending(cell: AdjectiveCell) -> Option<&'static str> {
    match (cell.form, cell.case, cell.number, cell.gender) {
        (
            AdjectiveForm::Short,
            Case::Nominative | Case::Vocative,
            Number::Singular,
            Gender::Feminine,
        ) => Some("и"),
        (
            AdjectiveForm::Long,
            Case::Nominative | Case::Vocative,
            Number::Singular,
            Gender::Feminine,
        ) => Some("иꙗ"),
        (
            AdjectiveForm::Short,
            Case::Nominative | Case::Vocative,
            Number::Plural,
            Gender::Masculine,
        ) => Some("е"),
        (
            AdjectiveForm::Long,
            Case::Nominative | Case::Vocative,
            Number::Plural,
            Gender::Masculine,
        ) => Some("еи"),
        _ => None,
    }
}

pub fn decline(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let normalized_lexeme = AdjectiveLexeme {
        lemma: crate::orthography::canonical_display(&lexeme.lemma)?,
        class: lexeme.class,
    };
    let lexeme = &normalized_lexeme;
    if let Some(identity) = LongOnlyAdjectiveIdentity::classify_source_union_lemma(&lexeme.lemma) {
        if identity.class() != lexeme.class {
            return Err(InflectionError::InvalidInput {
                reason: "the reviewed long-only adjective identity has a contradictory class"
                    .to_string(),
            });
        }
        return decline_long_only(identity, cell);
    }
    let (stem, long_only) = adjective_citation_stem(&lexeme.lemma, lexeme.class)?;
    if long_only && cell.form != AdjectiveForm::Long {
        return Err(InflectionError::historically_invalid(
            &lexeme.lemma,
            crate::RequestedCell::Adjective(cell),
        ));
    }
    decline_validated_stem(stem, lexeme.class, cell, &lexeme.lemma)
}

/// Decline an adjective when the caller explicitly supplies whether the lemma
/// itself is a short or long citation.
///
/// This strict path is useful for lexical categories such as determiners that
/// fix one adjectival realization. In particular, it disambiguates soft `-ии`:
/// a caller can state that it is a long citation instead of relying on lexical
/// identity or spelling inference.
pub fn decline_from_citation(
    lexeme: &AdjectiveLexeme,
    citation_form: AdjectiveForm,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    if cell.form != citation_form {
        return Err(InflectionError::InvalidInput {
            reason: "the requested adjective form contradicts the explicit citation form"
                .to_string(),
        });
    }
    let lemma = crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(identity) = LongOnlyAdjectiveIdentity::classify_source_union_lemma(&lemma) {
        if identity.class() != lexeme.class || citation_form != AdjectiveForm::Long {
            return Err(InflectionError::InvalidInput {
                reason:
                    "the reviewed long-only adjective identity has contradictory citation metadata"
                        .to_string(),
            });
        }
        return decline_long_only(identity, cell);
    }
    let stem = match (lexeme.class, citation_form) {
        (AdjectiveClass::Hard, AdjectiveForm::Short) => {
            strip_citation(&lemma, &["ъ"], "hard short")?
        }
        (AdjectiveClass::Soft, AdjectiveForm::Short) => {
            strip_citation(&lemma, &["ь", "и"], "soft short")?
        }
        (AdjectiveClass::Hard, AdjectiveForm::Long) => {
            strip_citation(&lemma, &["ꙑи", "ыи"], "hard long")?
        }
        (AdjectiveClass::Soft, AdjectiveForm::Long) => {
            strip_citation(&lemma, &["ии"], "soft long")?
        }
    };
    decline_validated_stem(stem, lexeme.class, cell, &lemma)
}

/// Declines an already selected adjective stem. Participles use this entry point so
/// adjective agreement has one implementation without pretending the verbal stem is
/// itself a dictionary adjective citation.
pub fn decline_stem(
    stem: &str,
    class: AdjectiveClass,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = crate::orthography::canonical_display(stem)?;
    if stem.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "an adjective agreement stem cannot be empty".to_string(),
        });
    }
    decline_validated_stem(&stem, class, cell, &stem)
}

/// Decline a soft adjective whose workstem ends in the suffix `ьj`.
///
/// The caller supplies the part of the workstem before `ьj`: `"бож"` for
/// source workstem `бож.ьj`, or `"трет"` for `трет.ьj`. This boundary cannot
/// be represented by ordinary [`AdjectiveClass::Soft`] concatenation. The yer
/// on the stem side surfaces as `и`, while the following terminal is resolved
/// after `j` (`а` -> `ꙗ`, `е` -> `ѥ`, `у` -> `ю`, and so on). Polivanova's
/// §§70, 72, and Table 304 explicitly derive the resulting `божии`, `божиѥ`,
/// `божиꙗ`, `божиѭ`, and `божиѩ` profile.
pub fn decline_j_stem(
    stem_before_yer_j: &str,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = crate::orthography::canonical_display(stem_before_yer_j)?;
    if stem.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "an adjective j-stem base cannot be empty".to_string(),
        });
    }
    let (ending, rule_id) = match cell.form {
        AdjectiveForm::Short => (soft_short_ending(cell), RuleId::AdjectiveSoftShort),
        AdjectiveForm::Long => (soft_long_ending(cell), RuleId::AdjectiveSoftLong),
    };
    let text = join_yer_j_boundary(&stem, ending);
    Ok(PredictedForm {
        text: text.clone(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: format!("{stem}.ьj"),
            after: text,
            reason: "attach the soft adjective terminal and resolve the ьj boundary",
        }],
    })
}

fn join_yer_j_boundary(stem: &str, ending: &str) -> String {
    let (boundary_vowel, rest) = if let Some(rest) = ending.strip_prefix("оу") {
        ("ю", rest)
    } else {
        let mut chars = ending.chars();
        let Some(first) = chars.next() else {
            return format!("{stem}и");
        };
        let replacement = match first {
            'ь' | 'и' => "и",
            'а' => "ꙗ",
            'е' => "ѥ",
            'ѣ' => "ꙗ",
            'ѧ' => "ѩ",
            'ѫ' => "ѭ",
            'у' => "ю",
            _ => return format!("{stem}и{ending}"),
        };
        (replacement, chars.as_str())
    };
    format!("{stem}и{boundary_vowel}{rest}")
}

fn decline_validated_stem(
    stem: &str,
    class: AdjectiveClass,
    cell: AdjectiveCell,
    before: &str,
) -> Result<PredictedForm, InflectionError> {
    let (ending, rule_id) = match (class, cell.form) {
        (AdjectiveClass::Hard, AdjectiveForm::Short) => {
            (hard_short_ending(cell), RuleId::AdjectiveHardShort)
        }
        (AdjectiveClass::Hard, AdjectiveForm::Long) => {
            (hard_long_ending(cell), RuleId::AdjectiveHardLong)
        }
        (AdjectiveClass::Soft, AdjectiveForm::Short) => {
            (soft_short_ending(cell), RuleId::AdjectiveSoftShort)
        }
        (AdjectiveClass::Soft, AdjectiveForm::Long) => {
            (soft_long_ending(cell), RuleId::AdjectiveSoftLong)
        }
    };
    let changed_stem = if class == AdjectiveClass::Hard
        && (ending.starts_with('ѣ') || matches!(ending, "и" | "ии"))
    {
        palatalize(stem, [('к', "ц"), ('г', "ѕ"), ('х', "с")])
    } else if class == AdjectiveClass::Hard && ending == "е" {
        palatalize(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")])
    } else {
        stem.to_string()
    };
    let text = format!("{changed_stem}{ending}");
    Ok(PredictedForm {
        text: text.clone(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: before.to_string(),
            after: text,
            reason: "attach the class and form specific adjective agreement ending",
        }],
    })
}

fn strip_citation<'a>(
    lemma: &'a str,
    endings: &[&str],
    class: &str,
) -> Result<&'a str, InflectionError> {
    endings
        .iter()
        .find_map(|ending| lemma.strip_suffix(ending))
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: format!("a {class} adjective citation has an incompatible ending"),
        })
}

fn strip_long_citation<'a>(lemma: &'a str, endings: &[&str]) -> Option<&'a str> {
    endings
        .iter()
        .find_map(|ending| lemma.strip_suffix(ending))
        .filter(|stem| !stem.is_empty())
}

fn adjective_citation_stem(
    lemma: &str,
    class: AdjectiveClass,
) -> Result<(&str, bool), InflectionError> {
    match class {
        AdjectiveClass::Hard => {
            if let Some(stem) = strip_long_citation(lemma, &["ꙑи", "ыи"]) {
                Ok((stem, true))
            } else {
                Ok((strip_citation(lemma, &["ъ"], "hard")?, false))
            }
        }
        // `-ии` is ambiguous: compare long-only прочии with short-only божии.
        // The exhaustive lexical inventory is routed before this generic path.
        AdjectiveClass::Soft => Ok((strip_citation(lemma, &["ь", "и"], "soft")?, false)),
    }
}

/// Since the phase-4 adjective merge the four shared ending tables live in
/// the merged kernel `church_slavonic_core::adjective`, queried with
/// `Recension::OldChurchSlavonic`; these shims keep this module's ending
/// vocabulary byte-identical. The kernel's totality test guarantees every
/// OCS cell is populated with exactly one ending.
fn kernel_ending(
    class: kernel::AdjectiveClass,
    form: AdjectiveForm,
    cell: AdjectiveCell,
) -> &'static str {
    let endings = match form {
        AdjectiveForm::Short => kernel::short_ending(
            class,
            cell.case,
            cell.number,
            cell.gender,
            cell.animacy,
            OCS,
        ),
        AdjectiveForm::Long => kernel::long_ending(
            class,
            cell.case,
            cell.number,
            cell.gender,
            cell.animacy,
            OCS,
        ),
    };
    endings.first().copied().unwrap_or_default()
}

fn hard_long_ending(cell: AdjectiveCell) -> &'static str {
    kernel_ending(kernel::AdjectiveClass::Hard, AdjectiveForm::Long, cell)
}

fn soft_short_ending(cell: AdjectiveCell) -> &'static str {
    kernel_ending(kernel::AdjectiveClass::Soft, AdjectiveForm::Short, cell)
}

fn soft_long_ending(cell: AdjectiveCell) -> &'static str {
    kernel_ending(kernel::AdjectiveClass::Soft, AdjectiveForm::Long, cell)
}

fn hard_short_ending(cell: AdjectiveCell) -> &'static str {
    kernel_ending(kernel::AdjectiveClass::Hard, AdjectiveForm::Short, cell)
}

fn palatalize<const N: usize>(stem: &str, replacements: [(char, &str); N]) -> String {
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let Some((_, replacement)) = replacements.iter().find(|(from, _)| *from == last) else {
        return stem.to_string();
    };
    let prefix_len = stem.len() - last.len_utf8();
    format!("{}{replacement}", &stem[..prefix_len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_short_agreement_includes_dual() {
        let mal = AdjectiveLexeme {
            lemma: "малъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let form = decline(
            &mal,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Dual,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        )
        .expect("supported");
        assert_eq!(form.text, "малѣ");
    }

    #[test]
    fn hard_short_velar_seams_palatalize() {
        let drug = AdjectiveLexeme {
            lemma: "дроугъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let form = decline(
            &drug,
            AdjectiveCell {
                case: Case::Dative,
                number: Number::Singular,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        )
        .expect("supported");
        assert_eq!(form.text, "дроуѕѣ");
    }

    #[test]
    fn long_and_soft_paradigms_are_distinct() {
        let hard = AdjectiveLexeme {
            lemma: "добръ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let long = decline(
            &hard,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
            },
        )
        .expect("hard long");
        assert_eq!(long.text, "добрꙑи");

        let soft = AdjectiveLexeme {
            lemma: "синь".to_string(),
            class: AdjectiveClass::Soft,
        };
        let soft_long = decline(
            &soft,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
            },
        )
        .expect("soft long");
        assert_eq!(soft_long.text, "синии");
    }

    #[test]
    fn yer_j_workstem_reproduces_the_reviewed_direct_profile() {
        let form = |case, number, gender, adjective_form| {
            decline_j_stem(
                "бож",
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: adjective_form,
                },
            )
            .expect("reviewed j-stem cell")
            .text
        };
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                AdjectiveForm::Short,
            ),
            "божии"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Neuter,
                AdjectiveForm::Short,
            ),
            "божиѥ"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                AdjectiveForm::Short,
            ),
            "божиꙗ"
        );
        assert_eq!(
            form(
                Case::Accusative,
                Number::Singular,
                Gender::Feminine,
                AdjectiveForm::Short,
            ),
            "божиѭ"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Plural,
                Gender::Feminine,
                AdjectiveForm::Short,
            ),
            "божиѩ"
        );
    }

    #[test]
    fn yer_j_workstem_remains_productive_in_long_cells() {
        let form = |case, gender| {
            decline_j_stem(
                "трет",
                AdjectiveCell {
                    case,
                    number: Number::Singular,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                },
            )
            .expect("productive j-stem long cell")
            .text
        };
        assert_eq!(form(Case::Nominative, Gender::Masculine), "третиии");
        assert_eq!(form(Case::Nominative, Gender::Neuter), "третиѥѥ");
        assert_eq!(form(Case::Genitive, Gender::Masculine), "третиꙗѥго");
        assert_eq!(form(Case::Dative, Gender::Masculine), "третиюѥмоу");
    }

    #[test]
    fn invalid_citations_and_hostile_lemmas_are_typed() {
        let cell = AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        };
        for lemma in ["", "добр ъ", "добръ\0", "ъ"] {
            let result = decline(
                &AdjectiveLexeme {
                    lemma: lemma.to_string(),
                    class: AdjectiveClass::Hard,
                },
                cell,
            );
            assert!(matches!(result, Err(InflectionError::InvalidInput { .. })));
        }
    }

    #[test]
    fn long_only_inventory_is_exhaustive_and_rejects_every_short_cell() {
        assert_eq!(LongOnlyAdjectiveIdentity::ALL.len(), 3);
        for identity in LongOnlyAdjectiveIdentity::ALL {
            let mut long = 0;
            let mut short = 0;
            for cell in AdjectiveCell::all() {
                match cell.form {
                    AdjectiveForm::Long => {
                        let form = decline_long_only(identity, cell)
                            .expect("every long-only adjective has a complete long paradigm");
                        assert!(!form.text.is_empty());
                        long += 1;
                    }
                    AdjectiveForm::Short => {
                        assert!(matches!(
                            decline_long_only(identity, cell),
                            Err(InflectionError::HistoricallyInvalidCell { .. })
                        ));
                        short += 1;
                    }
                }
            }
            assert_eq!((long, short), (126, 126));
        }
    }

    #[test]
    fn prochii_profile_matches_polivanova_section_304() {
        let identity = LongOnlyAdjectiveIdentity::OtherProchii;
        for (case, number, gender, expected) in [
            (
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                "прочии",
            ),
            (Case::Nominative, Number::Singular, Gender::Neuter, "прочеѥ"),
            (
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                "прочаꙗ",
            ),
            (
                Case::Accusative,
                Number::Singular,
                Gender::Feminine,
                "прочѫѭ",
            ),
            (
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
                "прочии",
            ),
            (Case::Nominative, Number::Plural, Gender::Neuter, "прочаꙗ"),
            (Case::Nominative, Number::Plural, Gender::Feminine, "прочѧѩ"),
            (
                Case::Accusative,
                Number::Plural,
                Gender::Masculine,
                "прочѧѩ",
            ),
        ] {
            let form = decline_long_only(
                identity,
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                },
            )
            .expect("source-profile cell");
            assert_eq!(form.text, expected);
        }
    }

    #[test]
    fn long_citations_are_parsed_without_inventing_short_lemmas() {
        for (lemma, class, expected) in [
            ("которꙑи", AdjectiveClass::Hard, "котории"),
            ("которыи", AdjectiveClass::Hard, "котории"),
            ("прокꙑи", AdjectiveClass::Hard, "проции"),
            ("прочии", AdjectiveClass::Soft, "прочии"),
        ] {
            let lexeme = AdjectiveLexeme {
                lemma: lemma.to_string(),
                class,
            };
            let long = decline(
                &lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                },
            )
            .expect("long citation");
            assert_eq!(long.text, expected);

            assert!(matches!(
                decline(
                    &lexeme,
                    AdjectiveCell {
                        form: AdjectiveForm::Short,
                        ..cell(
                            AdjectiveForm::Long,
                            Case::Nominative,
                            Number::Plural,
                            Gender::Masculine,
                            Animacy::Inanimate,
                        )
                    }
                ),
                Err(InflectionError::HistoricallyInvalidCell { .. })
            ));
        }
    }

    #[test]
    fn long_only_aliases_are_exhaustive_and_nonoverlapping() {
        let mut aliases = std::collections::BTreeSet::new();
        for identity in LongOnlyAdjectiveIdentity::ALL {
            assert_eq!(
                LongOnlyAdjectiveIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate alias {alias}");
                assert_eq!(
                    LongOnlyAdjectiveIdentity::classify_source_union_lemma(alias),
                    Some(identity)
                );
            }
        }
        assert_eq!(aliases.len(), 5);
    }

    #[test]
    fn productive_comparison_rejects_a_long_only_citation() {
        let result = productive_new_comparative(&AdjectiveLexeme {
            lemma: "прочии".to_string(),
            class: AdjectiveClass::Soft,
        });
        assert!(matches!(result, Err(InflectionError::InvalidInput { .. })));
    }

    #[test]
    fn explicit_citation_form_disambiguates_soft_ii() {
        let cell = AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Long,
        };
        assert_eq!(
            decline_from_citation(
                &AdjectiveLexeme {
                    lemma: "синии".to_string(),
                    class: AdjectiveClass::Soft,
                },
                AdjectiveForm::Long,
                cell,
            )
            .expect("explicit long citation")
            .text,
            "синии"
        );
        assert!(matches!(
            decline_from_citation(
                &AdjectiveLexeme {
                    lemma: "синии".to_string(),
                    class: AdjectiveClass::Soft,
                },
                AdjectiveForm::Short,
                cell,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    #[test]
    fn masculine_accusative_animacy_is_not_collapsed() {
        let adjective = AdjectiveLexeme {
            lemma: "добръ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let cell = AdjectiveCell {
            case: Case::Accusative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        };
        assert_eq!(decline(&adjective, cell).expect("inanimate").text, "добръ");
        assert_eq!(
            decline(
                &adjective,
                AdjectiveCell {
                    animacy: Animacy::Animate,
                    ..cell
                }
            )
            .expect("animate")
            .text,
            "добра"
        );
    }

    fn cell(
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> AdjectiveCell {
        AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        }
    }

    #[test]
    fn productive_new_comparative_forms_velar_and_nonvelar_principal_parts() {
        for (lemma, expected_syncopated) in [
            ("новъ", "новѣи"),
            ("горькъ", "горьчаи"),
            ("драгъ", "дражаи"),
            ("тихъ", "тишаи"),
        ] {
            let comparative = productive_new_comparative(&AdjectiveLexeme {
                lemma: lemma.to_string(),
                class: AdjectiveClass::Hard,
            })
            .expect("productive new comparative");
            assert_eq!(comparative.syncopated_citation, expected_syncopated);
            assert_eq!(
                comparative.expanded_citation,
                format!("{expected_syncopated}ши")
            );
        }
    }

    #[test]
    fn new_comparative_has_all_syncopated_and_alien_terminal_cells() {
        let new = productive_new_comparative(&AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        })
        .expect("productive new comparative");
        let examples = [
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣи",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Neuter,
                    Animacy::Inanimate,
                ),
                "новѣѥ",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣии",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                ),
                "новѣиши",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                ),
                "новѣишиꙗ",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Plural,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣише",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Plural,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣишеи",
            ),
        ];
        for (cell, expected) in examples {
            assert_eq!(
                decline_comparative(&new, cell)
                    .expect("source-described comparative cell")
                    .text,
                expected
            );
        }
    }

    #[test]
    fn old_comparative_uses_its_independent_softened_principal_parts() {
        let old = ComparativeLexeme {
            positive_lemma: "грѫбъ".to_string(),
            syncopated_citation: "грѫбл҄ь".to_string(),
            expanded_citation: "грѫбл҄ьши".to_string(),
            formation: ComparativeFormation::Old,
        };
        for (cell, expected) in [
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "грѫбл҄ь",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Neuter,
                    Animacy::Inanimate,
                ),
                "грѫбл҄е",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "грѫбл҄ии",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "грѫбл҄ьша",
            ),
        ] {
            let predicted = decline_comparative(&old, cell).expect("old comparative");
            assert_eq!(predicted.text, expected);
            assert_eq!(predicted.rule_id, RuleId::AdjectiveComparativeOld);
        }
    }

    #[test]
    fn comparative_inventory_is_exhaustive_and_keeps_accusative_animacy() {
        let comparative = productive_new_comparative(&AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        })
        .expect("new comparative");
        let forms = AdjectiveCell::all()
            .map(|cell| decline_comparative(&comparative, cell).expect("complete cell"))
            .collect::<Vec<_>>();
        assert_eq!(forms.len(), 252);

        let inanimate = decline_comparative(
            &comparative,
            cell(
                AdjectiveForm::Short,
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
            ),
        )
        .expect("inanimate accusative");
        let animate = decline_comparative(
            &comparative,
            cell(
                AdjectiveForm::Short,
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
            ),
        )
        .expect("animate accusative");
        assert_eq!(inanimate.text, "новѣи");
        assert_eq!(animate.text, "новѣиша");
    }

    #[test]
    fn contradictory_comparative_principal_parts_are_rejected() {
        for lexeme in [
            ComparativeLexeme {
                positive_lemma: "новъ".to_string(),
                syncopated_citation: "новѣь".to_string(),
                expanded_citation: "новѣиши".to_string(),
                formation: ComparativeFormation::New,
            },
            ComparativeLexeme {
                positive_lemma: "грѫбъ".to_string(),
                syncopated_citation: "грѫбл҄ь".to_string(),
                expanded_citation: "грѫбьши".to_string(),
                formation: ComparativeFormation::Old,
            },
        ] {
            assert!(matches!(
                decline_comparative(
                    &lexeme,
                    cell(
                        AdjectiveForm::Short,
                        Case::Nominative,
                        Number::Singular,
                        Gender::Masculine,
                        Animacy::Inanimate,
                    ),
                ),
                Err(InflectionError::InvalidInput { .. })
            ));
        }
    }
}
