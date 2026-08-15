//! Conservative, source-cell-based verb principal-part extraction.

use crate::schema::{FormRow, HeadTemplate, LexemeRow, Registry, VerbMetadataRow};
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::{
    AoristFormation, FiniteTense, FiniteVerbCell, ImperativeCell, ImperativeFormation,
    ImperfectFormation, ImperfectVariantPolicy, LParticipleCell, VerbClass,
    orthography::{Script, detect_script},
};
use std::collections::{BTreeMap, BTreeSet};

pub const DICTIONARY_AUTHORITY: &str = "wiktionary-kaikki-2026-07-06";
pub const PRINCIPAL_PART_PROVENANCE: &str = "dictionary-principal-part";
pub const HEADWORD_PROVENANCE: &str = "dictionary-headword-metadata";
pub const IMPERFECT_VARIANT_AUTHORITY: &str =
    "UT OCS Online lesson 1 §4.2 https://lrc.la.utexas.edu/eieol/ocsol/10#grammar_967";

type ExcludedFeatures = BTreeSet<(String, String)>;

/// Derive all safe metadata rows. Exclusions are used by held-cell evaluation and
/// are applied before either a diagnostic or a cross-check can see a source cell.
pub fn derive(
    registry: &Registry,
    excluded: &ExcludedFeatures,
) -> Result<Vec<VerbMetadataRow>, String> {
    let grouped = grouped_forms(registry, excluded);
    let mut out = Vec::new();
    for lexeme in registry.lexemes.iter().filter(|row| row.pos == "verb") {
        derive_aspect(lexeme, &mut out)?;
        if detect_script(&lexeme.lemma) != Script::Cyrillic {
            continue;
        }
        let Some(class) = parse_class(&lexeme.class) else {
            continue;
        };
        derive_present(lexeme, class, &grouped, &mut out)?;
        derive_imperfect(lexeme, class, &grouped, &mut out)?;
        derive_new_aorist(lexeme, class, &grouped, &mut out)?;
        derive_imperative(lexeme, class, &grouped, &mut out)?;
        derive_l_participle(lexeme, class, &grouped, &mut out)?;
        derive_participles(lexeme, class, &grouped, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn grouped_forms<'a>(
    registry: &'a Registry,
    excluded: &ExcludedFeatures,
) -> BTreeMap<(String, String), Vec<&'a FormRow>> {
    let mut grouped: BTreeMap<(String, String), Vec<&FormRow>> = BTreeMap::new();
    for row in &registry.forms {
        if !excluded.contains(&(row.lexeme_id.clone(), row.feature.clone())) {
            grouped
                .entry((row.lexeme_id.clone(), row.feature.clone()))
                .or_default()
                .push(row);
        }
    }
    grouped
}

fn derive_aspect(lexeme: &LexemeRow, out: &mut Vec<VerbMetadataRow>) -> Result<(), String> {
    let templates: Vec<HeadTemplate> = serde_json::from_str(&lexeme.head_templates)
        .map_err(|error| format!("invalid head templates for {}: {error}", lexeme.id))?;
    let mut values = Vec::new();
    for template in &templates {
        let candidate = match template.name.as_str() {
            "cu-verb" => template.args.get("1"),
            "head" => template.args.get("g"),
            _ => None,
        };
        if let Some(value) = candidate.and_then(|value| aspect_code(value)) {
            if !values.contains(&value) {
                values.push(value);
            }
        }
    }
    if values.len() == 1 {
        out.push(row(
            lexeme,
            "aspect",
            0,
            "aspect",
            values[0],
            (HEADWORD_PROVENANCE, "headword:aspect", values[0], &[]),
        ));
    }
    Ok(())
}

fn aspect_code(value: &str) -> Option<&'static str> {
    match value {
        "impf" | "imperfective" => Some("imperfective"),
        "pf" | "perfective" => Some("perfective"),
        "biasp" | "biaspectual" => Some("biaspectual"),
        _ => None,
    }
}

fn derive_present(
    lexeme: &LexemeRow,
    class: VerbClass,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String> {
    let source_feature = "verb:finite:present:2:sg";
    let Some(default_sources) = forms(grouped, lexeme, source_feature) else {
        return Ok(());
    };
    let first_feature = "verb:finite:present:1:sg";
    let first_sources = forms(grouped, lexeme, first_feature).unwrap_or_default();
    let mut analyses = Vec::new();
    for default_source in default_sources {
        let Some(stem) = strip_present_2sg(class, &default_source.form) else {
            continue;
        };
        let first_candidates = first_sources
            .iter()
            .filter_map(|source| {
                strip_present_1sg(class, &source.form).map(|value| (*source, value))
            })
            .collect::<Vec<_>>();
        if needs_first_singular(class) && first_candidates.is_empty() {
            continue;
        }
        if first_candidates.is_empty() {
            analyses.push((default_source, stem, None));
        } else {
            analyses.extend(
                first_candidates
                    .into_iter()
                    .map(|(source, first)| (default_source, stem.clone(), Some((source, first)))),
            );
        }
    }
    analyses.sort_by_key(|(default, _, first)| {
        (
            default.rank,
            first.as_ref().map_or(0, |(source, _)| source.rank),
        )
    });
    analyses.dedup_by(|left, right| left.1 == right.1 && left.2 == right.2);
    for (analysis_rank, (default_source, stem, first)) in analyses.into_iter().enumerate() {
        let mut verb = VerbLexeme::new(&lexeme.lemma, class);
        verb.stems.present = Some(stem.clone());
        verb.stems.present_first_singular = first.as_ref().map(|(_, value)| value.clone());
        let crosschecks = finite_crosschecks(
            grouped,
            lexeme,
            &verb,
            FiniteTense::Present,
            &[source_feature, first_feature],
        );
        if crosschecks.is_none() {
            continue;
        }
        let rank = u16::try_from(analysis_rank).map_err(|_| "too many present analyses")?;
        out.push(row(
            lexeme,
            "present",
            rank,
            "class",
            class.code(),
            (HEADWORD_PROVENANCE, "headword:class", &lexeme.class, &[]),
        ));
        out.push(row(
            lexeme,
            "present",
            rank,
            "stem",
            &stem,
            (
                PRINCIPAL_PART_PROVENANCE,
                source_feature,
                &default_source.form,
                &crosschecks.clone().unwrap_or_default(),
            ),
        ));
        if let Some((source, first_stem)) = first {
            out.push(row(
                lexeme,
                "present",
                rank,
                "first-singular-stem",
                &first_stem,
                (PRINCIPAL_PART_PROVENANCE, first_feature, &source.form, &[]),
            ));
        }
    }
    Ok(())
}

fn derive_imperfect(
    lexeme: &LexemeRow,
    class: VerbClass,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String> {
    let source_feature = "verb:finite:imperfect:1:sg";
    let Some(sources) = forms(grouped, lexeme, source_feature) else {
        return Ok(());
    };
    let mut accepted = Vec::new();
    for source in sources {
        let Some((stem, formation)) = strip_imperfect_1sg(&source.form) else {
            continue;
        };
        let mut verb = VerbLexeme::new(&lexeme.lemma, class);
        verb.stems.imperfect = Some(stem.clone());
        verb.formations.imperfect = Some(formation);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
        let Some(crosschecks) = finite_crosschecks(
            grouped,
            lexeme,
            &verb,
            FiniteTense::Imperfect,
            &[source_feature],
        ) else {
            continue;
        };
        accepted.push((source, stem, formation, crosschecks));
    }
    for (rank, (source, stem, formation, crosschecks)) in accepted.into_iter().enumerate() {
        let rank = u16::try_from(rank).map_err(|_| "too many imperfect analyses")?;
        out.push(row(
            lexeme,
            "imperfect",
            rank,
            "stem",
            &stem,
            (
                PRINCIPAL_PART_PROVENANCE,
                &source.feature,
                &source.form,
                &crosschecks,
            ),
        ));
        out.push(row(
            lexeme,
            "imperfect",
            rank,
            "formation",
            FormationCode::from(formation).0,
            (
                PRINCIPAL_PART_PROVENANCE,
                &source.feature,
                &source.form,
                &crosschecks,
            ),
        ));
        let mut policy = row(
            lexeme,
            "imperfect",
            rank,
            "variant-policy",
            "uncontracted-only",
            (
                "curated-grammar-override",
                &source.feature,
                &source.form,
                &crosschecks,
            ),
        );
        policy.authority = IMPERFECT_VARIANT_AUTHORITY.to_string();
        out.push(policy);
    }
    Ok(())
}

fn derive_new_aorist(
    lexeme: &LexemeRow,
    class: VerbClass,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String> {
    let source_feature = "verb:finite:aorist:1:sg";
    let Some(sources) = forms(grouped, lexeme, source_feature) else {
        return Ok(());
    };
    let mut accepted = Vec::new();
    for source in sources {
        let Some(stem) = source
            .form
            .strip_suffix("охъ")
            .filter(|stem| !stem.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        let mut verb = VerbLexeme::new(&lexeme.lemma, class);
        verb.stems.aorist = Some(stem.clone());
        verb.formations.aorist = Some(AoristFormation::New);
        let Some(crosschecks) = finite_crosschecks(
            grouped,
            lexeme,
            &verb,
            FiniteTense::Aorist,
            &[source_feature],
        ) else {
            continue;
        };
        accepted.push((source, stem, AoristFormation::New, crosschecks));
    }
    emit_stem_formations(lexeme, "aorist", accepted, out)
}

fn derive_imperative(
    lexeme: &LexemeRow,
    class: VerbClass,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String> {
    let source_feature = "verb:imperative:2:sg";
    let Some(sources) = forms(grouped, lexeme, source_feature) else {
        return Ok(());
    };
    let diagnostic_features = [
        "verb:imperative:1:du",
        "verb:imperative:2:du",
        "verb:imperative:1:pl",
        "verb:imperative:2:pl",
    ];
    let mut analysis_rank = 0_u16;
    for source in sources {
        let Some(stem) = source
            .form
            .strip_suffix('и')
            .filter(|stem| !stem.is_empty())
            .map(str::to_string)
        else {
            continue;
        };
        for formation in [ImperativeFormation::ISeries, ImperativeFormation::YatSeries] {
            let mut verb = VerbLexeme::new(&lexeme.lemma, class);
            verb.stems.imperative = Some(stem.clone());
            verb.formations.imperative = Some(formation);
            let mut crosschecks = Vec::new();
            let mut formation_source = None;
            let mut contradiction = false;
            for feature in diagnostic_features {
                let Some(expected) = forms(grouped, lexeme, feature) else {
                    continue;
                };
                let Some(cell) = parse_imperative_cell(feature) else {
                    return Err(format!("invalid internal imperative feature {feature}"));
                };
                let predicted = old_church_slavonic_core::verb::imperative(&verb, cell)
                    .map_err(|error| format!("imperative cross-check failed: {error}"))?;
                if let Some(matched) = expected.iter().find(|row| row.form == predicted.text) {
                    crosschecks.push(feature);
                    formation_source.get_or_insert(*matched);
                } else {
                    contradiction = true;
                }
            }
            let Some(formation_source) = formation_source else {
                continue;
            };
            if contradiction {
                continue;
            }
            out.push(row(
                lexeme,
                "imperative",
                analysis_rank,
                "stem",
                &stem,
                (
                    PRINCIPAL_PART_PROVENANCE,
                    source_feature,
                    &source.form,
                    &crosschecks,
                ),
            ));
            out.push(row(
                lexeme,
                "imperative",
                analysis_rank,
                "formation",
                imperative_code(formation),
                (
                    PRINCIPAL_PART_PROVENANCE,
                    &formation_source.feature,
                    &formation_source.form,
                    &crosschecks,
                ),
            ));
            analysis_rank = analysis_rank
                .checked_add(1)
                .ok_or("too many imperative analyses")?;
        }
    }
    Ok(())
}

fn derive_l_participle(
    lexeme: &LexemeRow,
    class: VerbClass,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String> {
    let source_feature = "verb:l-participle:m:sg";
    let Some(sources) = forms(grouped, lexeme, source_feature) else {
        return Ok(());
    };
    let mut rank = 0_u16;
    for source in sources {
        let Some(stem) = source
            .form
            .strip_suffix("лъ")
            .filter(|stem| !stem.is_empty())
        else {
            continue;
        };
        let mut verb = VerbLexeme::new(&lexeme.lemma, class);
        verb.stems.l_participle = Some(stem.to_string());
        let Some(crosschecks) = l_participle_crosschecks(grouped, lexeme, &verb, source_feature)
        else {
            continue;
        };
        out.push(row(
            lexeme,
            "l-participle",
            rank,
            "stem",
            stem,
            (
                PRINCIPAL_PART_PROVENANCE,
                source_feature,
                &source.form,
                &crosschecks,
            ),
        ));
        rank = rank
            .checked_add(1)
            .ok_or("too many l-participle analyses")?;
    }
    Ok(())
}

fn derive_participles(
    lexeme: &LexemeRow,
    class: VerbClass,
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String> {
    for (system, feature) in [
        (
            "present-active-participle",
            "verb:participle:present-active:citation",
        ),
        (
            "present-passive-participle",
            "verb:participle:present-passive:citation",
        ),
        (
            "past-active-participle",
            "verb:participle:past-active:citation",
        ),
        (
            "past-passive-participle",
            "verb:participle:past-passive:citation",
        ),
    ] {
        let Some(sources) = forms(grouped, lexeme, feature) else {
            continue;
        };
        let mut rank = 0_u16;
        for source in sources {
            let derived = match system {
                "present-active-participle" => strip_present_active(class, &source.form),
                "present-passive-participle" => strip_present_passive(&source.form),
                "past-active-participle" => strip_past_active(&source.form),
                "past-passive-participle" => strip_past_passive(&source.form),
                _ => None,
            };
            let Some((stem, formation)) = derived else {
                continue;
            };
            out.push(row(
                lexeme,
                system,
                rank,
                "stem",
                &stem,
                (PRINCIPAL_PART_PROVENANCE, feature, &source.form, &[]),
            ));
            out.push(row(
                lexeme,
                system,
                rank,
                "formation",
                formation,
                (PRINCIPAL_PART_PROVENANCE, feature, &source.form, &[]),
            ));
            rank = rank.checked_add(1).ok_or("too many participle analyses")?;
        }
    }
    Ok(())
}

fn emit_stem_formations<F: Copy>(
    lexeme: &LexemeRow,
    system: &str,
    accepted: Vec<(&FormRow, String, F, Vec<&str>)>,
    out: &mut Vec<VerbMetadataRow>,
) -> Result<(), String>
where
    FormationCode: From<F>,
{
    for (rank, (source, stem, formation, crosschecks)) in accepted.into_iter().enumerate() {
        let rank = u16::try_from(rank).map_err(|_| format!("too many {system} analyses"))?;
        out.push(row(
            lexeme,
            system,
            rank,
            "stem",
            &stem,
            (
                PRINCIPAL_PART_PROVENANCE,
                &source.feature,
                &source.form,
                &crosschecks,
            ),
        ));
        let formation = FormationCode::from(formation);
        out.push(row(
            lexeme,
            system,
            rank,
            "formation",
            formation.0,
            (
                PRINCIPAL_PART_PROVENANCE,
                &source.feature,
                &source.form,
                &crosschecks,
            ),
        ));
    }
    Ok(())
}

struct FormationCode(&'static str);

impl From<ImperfectFormation> for FormationCode {
    fn from(value: ImperfectFormation) -> Self {
        Self(match value {
            ImperfectFormation::A => "a",
            ImperfectFormation::YatA => "yat-a",
            ImperfectFormation::PalatalizedA => "palatalized-a",
            ImperfectFormation::PresentA => "present-a",
            ImperfectFormation::PresentYatA => "present-yat-a",
        })
    }
}

impl From<AoristFormation> for FormationCode {
    fn from(value: AoristFormation) -> Self {
        Self(match value {
            AoristFormation::Asigmatic => "asigmatic",
            AoristFormation::SigmaticPrimary => "sigmatic-primary",
            AoristFormation::SigmaticSecondary => "sigmatic-secondary",
            AoristFormation::SigmaticVowel => "sigmatic-vowel",
            AoristFormation::New => "new",
        })
    }
}

fn finite_crosschecks(
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    lexeme: &LexemeRow,
    verb: &VerbLexeme,
    tense: FiniteTense,
    source_features: &[&str],
) -> Option<Vec<&'static str>> {
    let mut checked = Vec::new();
    for cell in finite_cells(tense) {
        let feature = finite_feature(cell);
        if source_features.contains(&feature) {
            continue;
        }
        let Some(expected) = forms(grouped, lexeme, feature) else {
            continue;
        };
        let Ok(predicted) = old_church_slavonic_core::verb::finite(verb, cell) else {
            return None;
        };
        if !expected.iter().any(|row| row.form == predicted.text) {
            return None;
        }
        checked.push(feature);
    }
    (!checked.is_empty()).then_some(checked)
}

fn l_participle_crosschecks(
    grouped: &BTreeMap<(String, String), Vec<&FormRow>>,
    lexeme: &LexemeRow,
    verb: &VerbLexeme,
    source_feature: &str,
) -> Option<Vec<&'static str>> {
    let mut checked = Vec::new();
    for number in old_church_slavonic_core::Number::ALL {
        for gender in old_church_slavonic_core::Gender::ALL {
            let cell = LParticipleCell { gender, number };
            let feature = l_participle_feature(cell);
            if feature == source_feature {
                continue;
            }
            let Some(expected) = forms(grouped, lexeme, feature) else {
                continue;
            };
            let Ok(predicted) = old_church_slavonic_core::verb::l_participle(verb, cell) else {
                return None;
            };
            if !expected.iter().any(|row| row.form == predicted.text) {
                return None;
            }
            checked.push(feature);
        }
    }
    (!checked.is_empty()).then_some(checked)
}

fn row(
    lexeme: &LexemeRow,
    system: &str,
    analysis_rank: u16,
    field: &str,
    value: &str,
    evidence: (&str, &str, &str, &[&str]),
) -> VerbMetadataRow {
    let (provenance, source_feature, source_form, crosschecks) = evidence;
    VerbMetadataRow {
        lexeme_id: lexeme.id.clone(),
        system: system.to_string(),
        analysis_rank,
        field: field.to_string(),
        value: value.to_string(),
        provenance: provenance.to_string(),
        source_feature: source_feature.to_string(),
        source_form: source_form.to_string(),
        crosscheck_features: crosschecks.join(" || "),
        authority: DICTIONARY_AUTHORITY.to_string(),
    }
}

fn forms<'a>(
    grouped: &BTreeMap<(String, String), Vec<&'a FormRow>>,
    lexeme: &LexemeRow,
    feature: &str,
) -> Option<Vec<&'a FormRow>> {
    grouped
        .get(&(lexeme.id.clone(), feature.to_string()))
        .cloned()
}

fn parse_class(value: &str) -> Option<VerbClass> {
    match value {
        "IA1" => Some(VerbClass::IA1),
        "IA2" => Some(VerbClass::IA2),
        "II1" => Some(VerbClass::II1),
        "II2" => Some(VerbClass::II2),
        "II3" => Some(VerbClass::II3),
        _ => None,
    }
}

fn strip_present_2sg(class: VerbClass, form: &str) -> Option<String> {
    let ending = if matches!(class, VerbClass::IA1 | VerbClass::IA2) {
        "еши"
    } else {
        "иши"
    };
    form.strip_suffix(ending)
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn strip_present_1sg(class: VerbClass, form: &str) -> Option<String> {
    let ending = if matches!(class, VerbClass::IA1 | VerbClass::IA2) {
        'ѫ'
    } else {
        'ѭ'
    };
    form.strip_suffix(ending)
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
}

fn needs_first_singular(class: VerbClass) -> bool {
    matches!(class, VerbClass::II1 | VerbClass::II2 | VerbClass::II3)
}

fn strip_imperfect_1sg(form: &str) -> Option<(String, ImperfectFormation)> {
    if let Some(stem) = form.strip_suffix("ѣахъ").filter(|stem| !stem.is_empty()) {
        return Some((stem.to_string(), ImperfectFormation::YatA));
    }
    let stem = form.strip_suffix("ахъ").filter(|stem| !stem.is_empty())?;
    // A palatalized `-аахъ` surface does not preserve its underlying velar.
    // Reject it rather than mislabeling the surface stem as an A formation.
    if stem
        .strip_suffix('а')
        .is_some_and(|base| base.ends_with(['ч', 'ж', 'ш']))
    {
        return None;
    }
    Some((stem.to_string(), ImperfectFormation::A))
}

fn strip_present_active(class: VerbClass, form: &str) -> Option<(String, &'static str)> {
    if let Some(stem) = form.strip_suffix('ѩ').filter(|stem| !stem.is_empty()) {
        return Some((stem.to_string(), "yusht-soft"));
    }
    if matches!(class, VerbClass::IA1 | VerbClass::IA2) {
        form.strip_suffix('ꙑ')
            .filter(|stem| !stem.is_empty())
            .map(|stem| (stem.to_string(), "yusht-hard"))
    } else {
        form.strip_suffix('ѧ')
            .filter(|stem| !stem.is_empty())
            .map(|stem| (stem.to_string(), "yesht-soft"))
    }
}

fn strip_present_passive(form: &str) -> Option<(String, &'static str)> {
    for (suffix, formation) in [
        ("ѥмъ", "iotated-em"),
        ("имъ", "im"),
        ("емъ", "em"),
        ("омъ", "om"),
    ] {
        if let Some(stem) = form.strip_suffix(suffix).filter(|stem| !stem.is_empty()) {
            return Some((stem.to_string(), formation));
        }
    }
    None
}

fn strip_past_active(form: &str) -> Option<(String, &'static str)> {
    if let Some(stem) = form.strip_suffix("въ").filter(|stem| !stem.is_empty()) {
        return Some((stem.to_string(), "vush"));
    }
    if let Some(stem) = form.strip_suffix('ь').filter(|stem| !stem.is_empty()) {
        return Some((stem.to_string(), "ish"));
    }
    form.strip_suffix('ъ')
        .filter(|stem| !stem.is_empty())
        .map(|stem| (stem.to_string(), "ush"))
}

fn strip_past_passive(form: &str) -> Option<(String, &'static str)> {
    for (suffix, formation) in [("енъ", "en"), ("нъ", "n"), ("тъ", "t")] {
        if let Some(stem) = form.strip_suffix(suffix).filter(|stem| !stem.is_empty()) {
            return Some((stem.to_string(), formation));
        }
    }
    None
}

fn imperative_code(value: ImperativeFormation) -> &'static str {
    match value {
        ImperativeFormation::ISeries => "i-series",
        ImperativeFormation::YatSeries => "yat-series",
    }
}

fn finite_cells(tense: FiniteTense) -> impl Iterator<Item = FiniteVerbCell> {
    old_church_slavonic_core::Number::ALL
        .into_iter()
        .flat_map(move |number| {
            old_church_slavonic_core::Person::ALL
                .into_iter()
                .map(move |person| FiniteVerbCell {
                    tense,
                    person,
                    number,
                })
        })
}

fn finite_feature(cell: FiniteVerbCell) -> &'static str {
    use old_church_slavonic_core::{Number::*, Person::*};
    match (cell.tense, cell.person, cell.number) {
        (FiniteTense::Present, First, Singular) => "verb:finite:present:1:sg",
        (FiniteTense::Present, Second, Singular) => "verb:finite:present:2:sg",
        (FiniteTense::Present, Third, Singular) => "verb:finite:present:3:sg",
        (FiniteTense::Present, First, Dual) => "verb:finite:present:1:du",
        (FiniteTense::Present, Second, Dual) => "verb:finite:present:2:du",
        (FiniteTense::Present, Third, Dual) => "verb:finite:present:3:du",
        (FiniteTense::Present, First, Plural) => "verb:finite:present:1:pl",
        (FiniteTense::Present, Second, Plural) => "verb:finite:present:2:pl",
        (FiniteTense::Present, Third, Plural) => "verb:finite:present:3:pl",
        (FiniteTense::Imperfect, First, Singular) => "verb:finite:imperfect:1:sg",
        (FiniteTense::Imperfect, Second, Singular) => "verb:finite:imperfect:2:sg",
        (FiniteTense::Imperfect, Third, Singular) => "verb:finite:imperfect:3:sg",
        (FiniteTense::Imperfect, First, Dual) => "verb:finite:imperfect:1:du",
        (FiniteTense::Imperfect, Second, Dual) => "verb:finite:imperfect:2:du",
        (FiniteTense::Imperfect, Third, Dual) => "verb:finite:imperfect:3:du",
        (FiniteTense::Imperfect, First, Plural) => "verb:finite:imperfect:1:pl",
        (FiniteTense::Imperfect, Second, Plural) => "verb:finite:imperfect:2:pl",
        (FiniteTense::Imperfect, Third, Plural) => "verb:finite:imperfect:3:pl",
        (FiniteTense::Aorist, First, Singular) => "verb:finite:aorist:1:sg",
        (FiniteTense::Aorist, Second, Singular) => "verb:finite:aorist:2:sg",
        (FiniteTense::Aorist, Third, Singular) => "verb:finite:aorist:3:sg",
        (FiniteTense::Aorist, First, Dual) => "verb:finite:aorist:1:du",
        (FiniteTense::Aorist, Second, Dual) => "verb:finite:aorist:2:du",
        (FiniteTense::Aorist, Third, Dual) => "verb:finite:aorist:3:du",
        (FiniteTense::Aorist, First, Plural) => "verb:finite:aorist:1:pl",
        (FiniteTense::Aorist, Second, Plural) => "verb:finite:aorist:2:pl",
        (FiniteTense::Aorist, Third, Plural) => "verb:finite:aorist:3:pl",
    }
}

fn l_participle_feature(cell: LParticipleCell) -> &'static str {
    use old_church_slavonic_core::{Gender::*, Number::*};
    match (cell.gender, cell.number) {
        (Masculine, Singular) => "verb:l-participle:m:sg",
        (Feminine, Singular) => "verb:l-participle:f:sg",
        (Neuter, Singular) => "verb:l-participle:n:sg",
        (Masculine, Dual) => "verb:l-participle:m:du",
        (Feminine, Dual) => "verb:l-participle:f:du",
        (Neuter, Dual) => "verb:l-participle:n:du",
        (Masculine, Plural) => "verb:l-participle:m:pl",
        (Feminine, Plural) => "verb:l-participle:f:pl",
        (Neuter, Plural) => "verb:l-participle:n:pl",
    }
}

fn parse_imperative_cell(feature: &str) -> Option<ImperativeCell> {
    use old_church_slavonic_core::{Number, Person};
    let parts = feature.split(':').collect::<Vec<_>>();
    let ["verb", "imperative", person, number] = parts.as_slice() else {
        return None;
    };
    Some(ImperativeCell {
        person: match *person {
            "1" => Person::First,
            "2" => Person::Second,
            "3" => Person::Third,
            _ => return None,
        },
        number: match *number {
            "sg" => Number::Singular,
            "du" => Number::Dual,
            "pl" => Number::Plural,
            _ => return None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_suffixes_are_strict() {
        assert_eq!(
            strip_present_2sg(VerbClass::IA1, "несеши").as_deref(),
            Some("нес")
        );
        assert_eq!(
            strip_present_1sg(VerbClass::II1, "правлѭ").as_deref(),
            Some("правл")
        );
        assert_eq!(
            strip_imperfect_1sg("несѣахъ"),
            Some(("нес".to_string(), ImperfectFormation::YatA))
        );
        assert_eq!(strip_imperfect_1sg("можаахъ"), None);
    }

    #[test]
    fn citation_participles_keep_formation_identity() {
        assert_eq!(
            strip_present_passive("несомъ"),
            Some(("нес".to_string(), "om"))
        );
        assert_eq!(
            strip_present_passive("плюѥмъ"),
            Some(("плю".to_string(), "iotated-em"))
        );
        assert_eq!(
            strip_past_active("правль"),
            Some(("правл".to_string(), "ish"))
        );
        assert_eq!(
            strip_past_passive("несенъ"),
            Some(("нес".to_string(), "en"))
        );
    }

    #[test]
    fn excluded_target_is_invisible_to_derivation() {
        let registry = Registry {
            lexemes: vec![LexemeRow {
                id: "нести|verb|fixture".to_string(),
                lemma: "нести".to_string(),
                page_word: "нести".to_string(),
                key: "нести".to_string(),
                pos: "verb".to_string(),
                class: "IA1".to_string(),
                raw_class: "IA1".to_string(),
                gender: String::new(),
                animacy: String::new(),
                number_restriction: String::new(),
                head_templates: "[]".to_string(),
                signature: "fixture".to_string(),
            }],
            aliases: Vec::new(),
            forms: vec![
                form("verb:finite:imperfect:1:sg", "несѣахъ"),
                form("verb:finite:imperfect:3:sg", "несѣаше"),
            ],
            verb_metadata: Vec::new(),
            overrides: Vec::new(),
        };
        let excluded = BTreeSet::from([(
            "нести|verb|fixture".to_string(),
            "verb:finite:imperfect:1:sg".to_string(),
        )]);
        assert!(
            derive(&registry, &excluded)
                .expect("safe exclusion")
                .iter()
                .all(|row| row.system != "imperfect")
        );
    }

    #[test]
    fn audited_fixture_derives_every_admitted_principal_part_system() {
        let registry = Registry {
            lexemes: vec![LexemeRow {
                id: "нести|verb|fixture".to_string(),
                lemma: "нести".to_string(),
                page_word: "нести".to_string(),
                key: "нести".to_string(),
                pos: "verb".to_string(),
                class: "IA1".to_string(),
                raw_class: "IA1".to_string(),
                gender: String::new(),
                animacy: String::new(),
                number_restriction: String::new(),
                head_templates: r#"[{"name":"cu-verb","args":{"1":"impf"}}]"#.to_string(),
                signature: "fixture".to_string(),
            }],
            aliases: Vec::new(),
            forms: [
                ("verb:finite:present:2:sg", "несеши"),
                ("verb:finite:present:3:sg", "несетъ"),
                ("verb:finite:imperfect:1:sg", "несѣахъ"),
                ("verb:finite:imperfect:3:sg", "несѣаше"),
                ("verb:finite:aorist:1:sg", "несохъ"),
                ("verb:finite:aorist:1:du", "несоховѣ"),
                ("verb:imperative:2:sg", "неси"),
                ("verb:imperative:1:du", "несивѣ"),
                ("verb:l-participle:m:sg", "неслъ"),
                ("verb:l-participle:f:sg", "несла"),
                ("verb:participle:present-active:citation", "несꙑ"),
                ("verb:participle:present-passive:citation", "несомъ"),
                ("verb:participle:past-active:citation", "несъ"),
                ("verb:participle:past-passive:citation", "несенъ"),
            ]
            .into_iter()
            .map(|(feature, value)| form(feature, value))
            .collect(),
            verb_metadata: Vec::new(),
            overrides: Vec::new(),
        };
        let rows = derive(&registry, &BTreeSet::new()).expect("safe fixture metadata");
        let systems = rows
            .iter()
            .map(|row| row.system.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            systems,
            BTreeSet::from([
                "aspect",
                "present",
                "imperfect",
                "aorist",
                "imperative",
                "l-participle",
                "present-active-participle",
                "present-passive-participle",
                "past-active-participle",
                "past-passive-participle",
            ])
        );
        assert!(rows.iter().all(|row| {
            row.provenance == PRINCIPAL_PART_PROVENANCE
                || row.provenance == HEADWORD_PROVENANCE
                || (row.field == "variant-policy"
                    && row.provenance == "curated-grammar-override"
                    && row.authority == IMPERFECT_VARIANT_AUTHORITY)
        }));
    }

    #[test]
    fn contradictory_crosscheck_rejects_an_analysis() {
        let mut registry = Registry {
            lexemes: vec![LexemeRow {
                id: "нести|verb|fixture".to_string(),
                lemma: "нести".to_string(),
                page_word: "нести".to_string(),
                key: "нести".to_string(),
                pos: "verb".to_string(),
                class: "IA1".to_string(),
                raw_class: "IA1".to_string(),
                gender: String::new(),
                animacy: String::new(),
                number_restriction: String::new(),
                head_templates: "[]".to_string(),
                signature: "fixture".to_string(),
            }],
            aliases: Vec::new(),
            forms: vec![
                form("verb:finite:imperfect:1:sg", "несѣахъ"),
                form("verb:finite:imperfect:3:sg", "противорѣчие"),
            ],
            verb_metadata: Vec::new(),
            overrides: Vec::new(),
        };
        let rows = derive(&registry, &BTreeSet::new()).expect("contradiction fails closed");
        assert!(rows.iter().all(|row| row.system != "imperfect"));
        registry.forms[1].form = "несѣаше".to_string();
        assert!(
            derive(&registry, &BTreeSet::new())
                .expect("consistent fixture")
                .iter()
                .any(|row| row.system == "imperfect")
        );
    }

    fn form(feature: &str, value: &str) -> FormRow {
        FormRow {
            lexeme_id: "нести|verb|fixture".to_string(),
            feature: feature.to_string(),
            rank: 0,
            form: value.to_string(),
            romanization: String::new(),
            source_spelling: value.to_string(),
            source_tags: String::new(),
        }
    }
}
