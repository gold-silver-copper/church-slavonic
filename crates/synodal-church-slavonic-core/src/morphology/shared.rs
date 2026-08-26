use crate::{
    AuthorityRole, Confidence, EpistemicRole, Error, Evidence, EvidenceId, EvidenceKind, FormSet,
    FormSource, FormVariant, GenerationPolicy, MetadataField, OrthographyProfile, Recension,
    Result, RuleId, RuleTrace, SourceId, SynodalWord, TraceStep,
};

pub(crate) fn required<T>(value: Option<&T>, field: MetadataField) -> Result<&T> {
    value.ok_or(Error::MissingPrincipalPart { field })
}

pub(crate) fn join(stem: &str, ending: &str) -> String {
    let mut text = String::with_capacity(stem.len() + ending.len());
    text.push_str(stem);
    text.push_str(ending);
    text
}

pub(crate) fn palatalize_final_velar(stem: &str) -> String {
    let replacement = match stem.chars().last() {
        Some('к') => Some('ч'),
        Some('г') => Some('ж'),
        Some('х') => Some('ш'),
        _ => None,
    };
    if let Some(replacement) = replacement {
        let mut value = stem.to_owned();
        value.pop();
        value.push(replacement);
        value
    } else {
        stem.to_owned()
    }
}

pub(crate) fn second_palatalize_final_velar(stem: &str) -> String {
    let replacement = match stem.chars().last() {
        Some('к') => Some('ц'),
        Some('г') => Some('з'),
        Some('х') => Some('с'),
        _ => None,
    };
    if let Some(replacement) = replacement {
        let mut value = stem.to_owned();
        value.pop();
        value.push(replacement);
        value
    } else {
        stem.to_owned()
    }
}

pub(crate) fn last_e_as_wide_e(stem: &str) -> String {
    let mut characters = stem.chars().collect::<Vec<_>>();
    if let Some(index) = characters.iter().rposition(|character| *character == 'е') {
        characters[index] = 'є';
    }
    characters.into_iter().collect()
}

pub(crate) fn last_o_as_omega(stem: &str) -> String {
    let mut characters = stem.chars().collect::<Vec<_>>();
    if let Some(index) = characters.iter().rposition(|character| *character == 'о') {
        characters[index] = 'ѡ';
    }
    characters.into_iter().collect()
}

pub(crate) fn normative(
    expanded: String,
    rule: &'static str,
    profile: OrthographyProfile,
    stage: &'static str,
    input: &str,
) -> Result<FormSet> {
    normative_variants(vec![expanded], rule, profile, stage, input)
}

pub(crate) fn normative_variants(
    expanded: Vec<String>,
    rule: &'static str,
    profile: OrthographyProfile,
    stage: &'static str,
    input: &str,
) -> Result<FormSet> {
    let rule_id = RuleId::from(rule);
    let evidence_id = EvidenceId::from(format!("normative:{rule}"));
    let evidence = Evidence {
        id: evidence_id.clone(),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: normative_citation(rule).into(),
        note: Some(format!("stable rule {rule}")),
    };
    let variants = expanded
        .into_iter()
        .map(|expanded| {
            let expanded = SynodalWord::parse(expanded)?.canonical().to_owned();
            let (accented, printed, warnings) = match profile {
                OrthographyProfile::Expanded => (None, expanded.clone(), Vec::new()),
                OrthographyProfile::ExpandedAccentless => {
                    let accentless = strip_presentation_marks(&expanded);
                    (
                        None,
                        accentless.clone(),
                        vec!["accent and breathing marks removed".into()],
                    )
                }
                OrthographyProfile::SynodalLiturgical => {
                    if !expanded.chars().any(is_accent_or_breathing) {
                        return Err(Error::OrthographicMetadataRequired {
                            field: MetadataField::AccentClass,
                        });
                    }
                    (Some(expanded.clone()), expanded.clone(), Vec::new())
                }
            };
            Ok(FormVariant {
                expanded: expanded.clone(),
                accented,
                printed: printed.clone(),
                romanization: None,
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                recension_mapping: None,
                confidence: Confidence::from_basis_points(9_500).unwrap_or(Confidence::CERTAIN),
                source: FormSource::SynodalNormativeGeneration {
                    rule: rule_id.clone(),
                },
                assumptions: vec![],
                evidence: vec![evidence.clone()],
                contradictions: vec![],
                warnings,
                rule_trace: RuleTrace::new(vec![TraceStep {
                    rule: rule_id.clone(),
                    stage: stage.into(),
                    input: input.into(),
                    output: printed,
                    source_recension: Some(Recension::SynodalRussian),
                    target_recension: Recension::SynodalRussian,
                    mapping: None,
                    evidence: vec![evidence_id.clone()],
                }]),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    FormSet::try_from_variants(variants)
}

pub(crate) fn normative_citation(rule: &str) -> &'static str {
    match rule {
        "SYN-NOUN-I-HARD-M-ALYPY-34"
        | "SYN-NOUN-I-HARD-VELAR-M-ALYPY-34"
        | "SYN-NOUN-I-MIXED-M-ALYPY-33-34"
        | "SYN-NOUN-I-HARD-N-ALYPY-34"
        | "SYN-NOUN-I-SOFT-M-ALYPY-34"
        | "SYN-NOUN-I-SOFT-N-ALYPY-34" => "Alypy (Gamanovich), §§34–38",
        "SYN-NOUN-I-MIXED-TS-M-ALYPY-8-33-37" => "Alypy (Gamanovich), §§8 and 33–37",
        "SYN-NOUN-I-U-STEM-M-ALYPY-37-38" => "Alypy (Gamanovich), §§37–38",
        "SYN-NOUN-I-HARD-M-IN-ETHNONYM-ALYPY-37"
        | "SYN-NOUN-I-SOFT-M-TEL-AGENT-ALYPY-37"
        | "SYN-NOUN-I-SOFT-N-ISHCHE-ALYPY-37"
        | "SYN-NOUN-INDECLINABLE-ALYPY-37" => "Alypy (Gamanovich), §37",
        "SYN-NOUN-I-SOFT-M-LORD-ALYPY-35-41" => "Alypy (Gamanovich), §§35, 38, and 41",
        "SYN-NOUN-I-M-UD-ES-ALYPY-44" => "Alypy (Gamanovich), §44 ꙋдъ : ꙋдес-",
        "SYN-NOUN-I-SOFT-J-M-ALYPY-34-37"
        | "SYN-NOUN-I-SOFT-EY-M-ALYPY-34-37"
        | "SYN-NOUN-I-SOFT-IE-N-ALYPY-34-37" => "Alypy (Gamanovich), §§34–37",
        "SYN-NOUN-II-HARD-ALYPY-39" | "SYN-NOUN-II-SOFT-ALYPY-39" => {
            "Alypy (Gamanovich), §§39–40, 44"
        }
        "SYN-NOUN-II-HARD-VELAR-ALYPY-39-40" | "SYN-NOUN-II-MIXED-ALYPY-39-40" => {
            "Alypy (Gamanovich), §§39–40"
        }
        "SYN-NOUN-II-SOFT-POSTVOCALIC-ANCIENT-PL-ALYPY-40"
        | "SYN-NOUN-II-SOFT-M-IA-ALYPY-39-40" => "Alypy (Gamanovich), §§39–40",
        "SYN-NOUN-II-SOFT-F-IA-ALYPY-32-39-40" => "Alypy (Gamanovich), §§32 and 39–40",
        "SYN-NOUN-III-F-ALYPY-41" | "SYN-NOUN-III-M-ALYPY-41" => "Alypy (Gamanovich), §41",
        "SYN-NOUN-IV-N-EN-ALYPY-42-43"
        | "SYN-NOUN-IV-N-ES-ALYPY-42-43"
        | "SYN-NOUN-IV-N-AT-ALYPY-42-43"
        | "SYN-NOUN-IV-F-ER-ALYPY-42-43" => "Alypy (Gamanovich), §§42–43",
        "SYN-NOUN-IV-N-ES-PAIRED-DUAL-ALYPY-44" => "Alypy (Gamanovich), §44 ѻко/ꙋхо",
        "SYN-NOUN-IV-N-ES-ALT-FIRST-ALYPY-42-44" => {
            "Alypy (Gamanovich), §§42–44 -ес- / first-declension alternatives"
        }
        "SYN-NOUN-IV-F-ER-DAUGHTER-ALYPY-42-44" => "Alypy (Gamanovich), §§42–44 дщи : дщер-",
        "SYN-NOUN-IV-F-OV-ALYPY-42-44"
        | "SYN-NOUN-IV-F-OV-SYNCOPATING-ALYPY-42-44"
        | "SYN-NOUN-IV-M-EN-ALYPY-42-44" => "Alypy (Gamanovich), §§42–44",
        "SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43" => "Alypy (Gamanovich), §43 камень notes",
        "SYN-NOUN-IV-M-EN-DAY-ALYPY-43" => "Alypy (Gamanovich), §43 день table",
        "SYN-ADJ-SHORT-HARD-ALYPY-53" | "SYN-ADJ-SHORT-SOFT-ALYPY-53" => {
            "Alypy (Gamanovich), §§53–55"
        }
        "SYN-ADJ-SHORT-VELAR-ALYPY-53-57" => "Alypy (Gamanovich), §§53–57",
        "SYN-ADJ-LONG-HARD-ALYPY-57" | "SYN-ADJ-LONG-SOFT-ALYPY-57" => {
            "Alypy (Gamanovich), §§56–57"
        }
        "SYN-ADJ-LONG-VELAR-ALYPY-57" => "Alypy (Gamanovich), §57 velar table",
        "SYN-ADJ-COMPARATIVE-LONG-ALYPY-58" => "Alypy (Gamanovich), §58",
        "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60" => {
            "Alypy (Gamanovich), §§58 and 60 short-comparison declension"
        }
        "SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59" => "Alypy (Gamanovich), §59",
        "SYN-ADJ-SUPERLATIVE-SHORT-PREDICATE-ALYPY-59-60-125-128" => {
            "Alypy (Gamanovich), §§59–60, 125, and 128"
        }
        "SYN-DETERMINER-HARD-ALYPY-45-48" => {
            "Alypy (Gamanovich), §§45 and 48 short and full determinative pronouns"
        }
        "SYN-DETERMINER-VES-MIXED-ALYPY-45-48" => {
            "Alypy (Gamanovich), §§45 and 48.7 весь mixed paradigm and no-dual restriction"
        }
        "SYN-DETERMINER-VSYAK-MIXED-ALYPY-45-48-57" => {
            "Alypy (Gamanovich), §§45, 48, and 57 всѧкъ/всѧкїй paradigms"
        }
        "SYN-DETERMINER-FULL-SK-ALYPY-45-57" => {
            "Alypy (Gamanovich), §§45 and 57 full -скїй declension and -ск-/-ст- alternation"
        }
        "SYN-NUMERAL-CARDINAL-ONE-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-TWO-BOTH-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-THREE-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-FOUR-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-I-STEM-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-TEN-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-HUNDRED-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-MAGNITUDE-NOUN-ALYPY-61-62" => {
            "Alypy (Gamanovich), §§61–62 simple cardinal and magnitude paradigms"
        }
        "SYN-NUMERAL-ORDINAL-ADJECTIVAL-ALYPY-68" => {
            "Alypy (Gamanovich), §68 ordinal formation and full-adjective declension"
        }
        "SYN-NUMERAL-COLLECTIVE-AGREEING-ALYPY-69"
        | "SYN-NUMERAL-COLLECTIVE-GOVERNING-ALYPY-69"
        | "SYN-NUMERAL-COLLECTIVE-HARD-PLURAL-ALYPY-69" => {
            "Alypy (Gamanovich), §69 collective numeral inventories and government"
        }
        "SYN-NUMERAL-MULTIPLICATIVE-ADJECTIVAL-ALYPY-61-70"
        | "SYN-NUMERAL-FRACTIONAL-NOUN-ALYPY-61-70" => {
            "Alypy (Gamanovich), §§61 and 70 multiplicative and fractional numerals"
        }
        "SYN-NUMERAL-FRACTIONAL-ADJECTIVAL-ALYPY-51-TARGET" => {
            "Alypy (Gamanovich), §51 full hard-adjective declension; Synodal Bible, III Esdras 14:11–12 полдесѧтый"
        }
        "SYN-PRONOUN-PERSONAL-FIRST-ALYPY-47"
        | "SYN-PRONOUN-PERSONAL-SECOND-ALYPY-47"
        | "SYN-PRONOUN-REFLEXIVE-ALYPY-47" => "Alypy (Gamanovich), §47 first group",
        "SYN-PRONOUN-THIRD-PERSON-ALYPY-46-47" => {
            "Alypy (Gamanovich), §§46–47 third-person paradigm"
        }
        "SYN-PRONOUN-SEI-ALYPY-45-48" => "Alypy (Gamanovich), §§45–48 сей/сій paradigm",
        "SYN-PRONOUN-SOFT-ALYPY-47-48" | "SYN-PRONOUN-HARD-ALYPY-47-48" => {
            "Alypy (Gamanovich), §§47–48 pronominal declension"
        }
        "SYN-PRONOUN-SOFT-I-ALTERNATING-ALYPY-45-48" => {
            "Alypy (Gamanovich), §§45–48 чій paradigm and і/ї spelling"
        }
        "SYN-PRONOUN-MIXED-POSSESSIVE-ALYPY-48" => {
            "Alypy (Gamanovich), §48 mixed possessive declension"
        }
        "SYN-PRONOUN-KII-ALYPY-48" => "Alypy (Gamanovich), §48 two-base кій paradigm",
        "SYN-PRONOUN-SHORT-HARD-ALYPY-48" => "Alypy (Gamanovich), §48 short-pronoun paradigm",
        "SYN-PRONOUN-SHORT-OV-MIXED-ALYPY-48" => {
            "Alypy (Gamanovich), §48 compound -ов- mixed paradigm"
        }
        "SYN-PRONOUN-SHORT-VELAR-ALYPY-48" | "SYN-PRONOUN-QUANTITY-VELAR-ALYPY-48" => {
            "Alypy (Gamanovich), §48 velar and quantity pronouns"
        }
        "SYN-PRONOUN-FULL-HARD-ALYPY-48-57"
        | "SYN-PRONOUN-FULL-SOFT-ALYPY-48-57"
        | "SYN-PRONOUN-FULL-VELAR-ALYPY-48-57" => {
            "Alypy (Gamanovich), §§48 and 57 full adjectival pronouns"
        }
        "SYN-PRONOUN-KTO-ALYPY-48" | "SYN-PRONOUN-CHTO-ALYPY-48" => {
            "Alypy (Gamanovich), §48 interrogative paradigms"
        }
        "SYN-PRONOUN-DERIVED-ALYPY-46-48" => "Alypy (Gamanovich), §§46–48 derived pronouns",
        "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48" => {
            "Alypy (Gamanovich), §48 negative-pronoun preposition interposition"
        }
        "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47" => {
            "Alypy (Gamanovich), §47 short-pronoun enclisis and accent"
        }
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47" => {
            "Alypy (Gamanovich), §47 на(н)и/въ(н)и contractions"
        }
        "SYN-VERB-PRESENT-ALYPY-80" => "Alypy (Gamanovich), §§79–80",
        "SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84" => {
            "Alypy (Gamanovich), §84 simple future of perfective verbs"
        }
        "SYN-VERB-AORIST-VOWEL-ALYPY-86" | "SYN-VERB-AORIST-CONSONANT-ALYPY-86" => {
            "Alypy (Gamanovich), §86"
        }
        "SYN-VERB-IMPERFECT-H-ALYPY-87"
        | "SYN-VERB-IMPERFECT-YAH-ALYPY-87"
        | "SYN-VERB-IMPERFECT-AH-ALYPY-87" => "Alypy (Gamanovich), §87",
        "SYN-VERB-IMPERATIVE-ALYPY-93" => "Alypy (Gamanovich), §93",
        "SYN-VERB-INFINITIVE-LEXICAL" => "Alypy (Gamanovich), §79; lexical infinitive",
        "SYN-VERB-REFLEXIVE-ALYPY-73" => "Alypy (Gamanovich), §73",
        "SYN-VERB-AORIST-VOWEL-T-ALYPY-86" => {
            "Alypy (Gamanovich), §86: the closed -тъ list in the 2nd/3rd singular aorist"
        }
        "SYN-ADJ-LONG-SIBILANT-ALYPY-57-58" => {
            "Alypy (Gamanovich), §§57–58 and 95–98; Synodal after-sibilant spelling"
        }
        "SYN-VERB-LPART-ALYPY-97" => "Alypy (Gamanovich), §97",
        "SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-ALYPY-95" => "Alypy (Gamanovich), §95",
        "SYN-VERB-PARTICIPLE-PAST-ACTIVE-ALYPY-96" => "Alypy (Gamanovich), §96",
        "SYN-VERB-PARTICIPLE-PRESENT-PASSIVE-ALYPY-99" => "Alypy (Gamanovich), §99",
        "SYN-VERB-PARTICIPLE-PAST-PASSIVE-ALYPY-100" => "Alypy (Gamanovich), §100",
        "SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98" => {
            "Alypy (Gamanovich), §95 citation forms and §98 complete declension"
        }
        "SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98" => {
            "Alypy (Gamanovich), §96 citation forms and §98 complete declension"
        }
        _ => "Synodal normative rule; see stable rule identifier",
    }
}

pub(crate) fn strip_presentation_marks(value: &str) -> String {
    value
        .chars()
        .filter(|character| !is_accent_or_breathing(*character))
        .collect()
}

pub(crate) fn is_accent_or_breathing(character: char) -> bool {
    matches!(
        character,
        '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}'
    )
}

/// Checks whether a generation policy may use the selected productive rule.
#[must_use]
pub const fn policy_allows_normative_rule(policy: GenerationPolicy) -> bool {
    matches!(
        policy,
        GenerationPolicy::Strict | GenerationPolicy::Productive | GenerationPolicy::Exploratory
    )
}
