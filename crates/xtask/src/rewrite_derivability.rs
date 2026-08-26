//! Rewrite-plan phase 3 groundwork: quantify how much of the extracted OCS
//! registry is derivable from the pure rule kernel plus compact per-lexeme
//! metadata (class codes, verb principal-part metadata), versus genuinely
//! irregular surface forms that must ship as lookup tables.
//!
//! For every attested cell in `data/extracted/forms.tsv` the harness computes a
//! rule-only prediction that deliberately bypasses the resolver's stored-form
//! precedence, then compares the predicted variant list to the stored surface
//! variants. Results land in `reports/rewrite-derivability.md`.

use old_church_slavonic::FormSet;
use old_church_slavonic::advanced::metadata as api_metadata;
use old_church_slavonic::advanced::rules;
use old_church_slavonic_core::adjective::{AdjectiveLexeme, LongOnlyAdjectiveIdentity};
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, Gender, NounCell, NounClass,
    Number, ParticipleCell, VerbClass, orthography,
};
use old_church_slavonic_extractor::extract::load_registry;
use old_church_slavonic_extractor::schema::{FormRow, LexemeRow};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

#[derive(Default)]
struct Counts {
    cells: usize,
    derivable: usize,
    divergent: usize,
    divergent_primary_match: usize,
    unsupported: usize,
    lexemes: usize,
    lexemes_fully_derivable: usize,
    residual_forms: usize,
    residual_form_bytes: usize,
    residual_romanization_bytes: usize,
}

enum Outcome {
    /// The rule kernel (plus per-lexeme metadata) reproduces the stored
    /// variant list exactly; the table rows carry no information.
    Derivable,
    /// The rules produce something else; the stored rows are irregular data.
    /// The flag records whether the primary (rank-0) variant still matches,
    /// i.e. the table row only contributes extra variants.
    Divergent { primary_match: bool },
    /// No rule prediction is possible (missing class metadata, closed-class
    /// table, unmodelled formation).
    Unsupported,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(extra) = args.next() {
        return Err(format!("rewrite-derivability takes no arguments, found {extra}").into());
    }
    let registry = load_registry(&root.join("data/extracted"))?;
    let grouped = crate::grouped_forms(&registry);

    let mut by_pos: BTreeMap<String, Counts> = BTreeMap::new();
    let mut divergent_categories: BTreeMap<String, usize> = BTreeMap::new();
    // First divergent example per category, for the report's example table.
    let mut divergent_examples: BTreeMap<String, String> = BTreeMap::new();
    let mut unsupported_categories: BTreeMap<String, usize> = BTreeMap::new();

    for lexeme in &registry.lexemes {
        let start = (lexeme.id.clone(), String::new());
        let end = (lexeme.id.clone(), "\u{10ffff}".to_string());
        let cells: Vec<(&str, &Vec<&FormRow>)> = grouped
            .range(start..=end)
            .map(|((_id, feature), rows)| (feature.as_str(), rows))
            .collect();
        if cells.is_empty() {
            continue;
        }
        let counts = by_pos.entry(lexeme.pos.clone()).or_default();
        counts.lexemes += 1;
        let mut all_derivable = true;
        let verb_metadata = (lexeme.pos == "verb")
            .then(|| api_metadata::verb_metadata_by_id(&lexeme.id).ok())
            .flatten();
        for (feature, rows) in cells {
            counts.cells += 1;
            let expected: Vec<&str> = rows.iter().map(|row| row.form.as_str()).collect();
            let (outcome, category, predicted) =
                classify_cell(lexeme, feature, &expected, verb_metadata.as_ref());
            if matches!(outcome, Outcome::Divergent { .. }) {
                divergent_examples.entry(category.clone()).or_insert_with(|| {
                    format!(
                        "{} `{}`: stored {} vs rules {}",
                        lexeme.lemma,
                        feature,
                        expected.join(" / "),
                        predicted.unwrap_or_default().join(" / "),
                    )
                });
            }
            match outcome {
                Outcome::Derivable => counts.derivable += 1,
                Outcome::Divergent { .. } | Outcome::Unsupported => {
                    match outcome {
                        Outcome::Divergent { primary_match } => {
                            counts.divergent += 1;
                            counts.divergent_primary_match += usize::from(primary_match);
                            *divergent_categories.entry(category).or_insert(0) += 1;
                        }
                        _ => {
                            counts.unsupported += 1;
                            *unsupported_categories.entry(category).or_insert(0) += 1;
                        }
                    }
                    all_derivable = false;
                    counts.residual_forms += rows.len();
                    counts.residual_form_bytes +=
                        rows.iter().map(|row| row.form.len()).sum::<usize>();
                    counts.residual_romanization_bytes +=
                        rows.iter().map(|row| row.romanization.len()).sum::<usize>();
                }
            }
        }
        if all_derivable {
            counts.lexemes_fully_derivable += 1;
        }
    }

    let markdown = render_markdown(
        &by_pos,
        &divergent_categories,
        &unsupported_categories,
        &divergent_examples,
    );
    fs::write(root.join("reports/rewrite-derivability.md"), &markdown)?;
    print!("{markdown}");
    Ok(())
}

/// Compute the rule-only prediction for one attested cell and compare it to
/// the stored variant list. Returns the outcome and a category label used to
/// attribute the residue.
fn classify_cell(
    lexeme: &LexemeRow,
    feature: &str,
    expected: &[&str],
    verb_metadata: Option<&api_metadata::DictionaryVerbMetadata>,
) -> (Outcome, String, Option<Vec<String>>) {
    let (predicted, category) = predict(lexeme, feature, verb_metadata);
    let outcome = match &predicted {
        Some(texts) if texts.iter().map(String::as_str).eq(expected.iter().copied()) => {
            Outcome::Derivable
        }
        Some(texts) => Outcome::Divergent {
            primary_match: texts.first().map(String::as_str) == expected.first().copied(),
        },
        None => Outcome::Unsupported,
    };
    (outcome, category, predicted)
}

fn predict(
    lexeme: &LexemeRow,
    feature: &str,
    verb_metadata: Option<&api_metadata::DictionaryVerbMetadata>,
) -> (Option<Vec<String>>, String) {
    match lexeme.pos.as_str() {
        "noun" => predict_noun(lexeme, feature),
        "adj" => predict_adjective(lexeme, feature),
        "verb" => predict_verb(lexeme, feature, verb_metadata),
        pos => (None, format!("closed-class/{pos}")),
    }
}

fn predict_noun(lexeme: &LexemeRow, feature: &str) -> (Option<Vec<String>>, String) {
    let category = format!(
        "noun/{}",
        if lexeme.class.is_empty() {
            "(no class)"
        } else {
            &lexeme.class
        }
    );
    let Some(cell) = crate::parse_noun_cell(feature) else {
        return (None, format!("noun/unparsed:{feature}"));
    };
    let Some(class) = parse_noun_class(&lexeme.class) else {
        return (None, category);
    };
    let Some(gender) = parse_gender(&lexeme.gender)
        .or_else(|| lexeme.gender.is_empty().then(|| class.intrinsic_gender()).flatten())
    else {
        return (None, category);
    };
    let animacy = parse_animacy(&lexeme.animacy);
    if animacy.is_none()
        && class.has_animacy_contrast()
        && gender == Gender::Masculine
        && cell.case == Case::Accusative
    {
        // Masculine accusatives are animacy-conditioned; without the fact the
        // rules cannot commit. Every other cell is animacy-independent.
        return (None, category);
    }
    let animacy = animacy.unwrap_or(Animacy::Inanimate);
    let noun = NounLexeme {
        lemma: lexeme.lemma.clone(),
        class,
        gender,
        animacy,
        number_restriction: crate::parse_restriction(&lexeme.number_restriction),
    };
    (
        single_prediction(old_church_slavonic_core::noun::decline(&noun, cell)),
        category,
    )
}

fn predict_adjective(lexeme: &LexemeRow, feature: &str) -> (Option<Vec<String>>, String) {
    if feature == "adj:comparative:citation" {
        // Comparative citations carry their own suffix-grade lexical fact; the
        // harness does not model them.
        return (None, "adj/comparative-citation".to_string());
    }
    let Some(cell) = crate::parse_adjective_cell(feature) else {
        return (None, format!("adj/unparsed:{feature}"));
    };
    if let Some(identity) = LongOnlyAdjectiveIdentity::classify_source_union_lemma(&lexeme.lemma) {
        return (
            form_set_prediction(rules::long_only_adjective(identity, cell)),
            format!("adj/long-only:{}", form_code(cell.form)),
        );
    }
    let class = match lexeme.class.as_str() {
        "adj-hard" => AdjectiveClass::Hard,
        "adj-soft" => AdjectiveClass::Soft,
        _ => return (None, format!("adj/{}", lexeme.class)),
    };
    let category = format!("adj/{}:{}", lexeme.class, form_code(cell.form));
    let adjective = AdjectiveLexeme {
        lemma: lexeme.lemma.clone(),
        class,
    };
    (
        single_prediction(old_church_slavonic_core::adjective::decline(&adjective, cell)),
        category,
    )
}

fn predict_verb(
    lexeme: &LexemeRow,
    feature: &str,
    metadata: Option<&api_metadata::DictionaryVerbMetadata>,
) -> (Option<Vec<String>>, String) {
    let parts: Vec<&str> = feature.split(':').collect();
    let category = match parts.as_slice() {
        ["verb", "finite", tense, ..] => format!("verb/finite:{tense}"),
        ["verb", "participle", kind, "citation"] => format!("verb/participle:{kind}"),
        ["verb", section, ..] => format!("verb/{section}"),
        _ => format!("verb/unparsed:{feature}"),
    };
    if matches!(parts.as_slice(), ["verb", "infinitive"] | ["verb", "supine"]) {
        // The infinitive is the citation form and the supine is derived from
        // it; both need only the lemma (plus the irregular-verb kernel).
        let class = crate::parse_productive_verb_class(&lexeme.class)
            .or_else(|| metadata.and_then(|m| m.present.first()).map(|a| a.class.value))
            .unwrap_or(VerbClass::Irregular);
        let verb = VerbLexeme::new(lexeme.lemma.clone(), class);
        let result = if parts[1] == "infinitive" {
            rules::infinitive_with(&verb)
        } else {
            rules::supine_with(&verb)
        };
        return (form_set_prediction(result), category);
    }
    let Some(metadata) = metadata else {
        return (None, category);
    };
    let result = match parts.as_slice() {
        ["verb", "finite", tense, person, number] => {
            let Some(cell) = crate::parse_finite_verb_cell(&format!(
                "verb:finite:{tense}:{person}:{number}"
            )) else {
                return (None, category);
            };
            api_metadata::finite_verb_from_dictionary_metadata(metadata, cell)
        }
        ["verb", "imperative", ..] => {
            let Some(cell) = crate::parse_imperative_cell(feature) else {
                return (None, category);
            };
            api_metadata::imperative_from_dictionary_metadata(metadata, cell)
        }
        ["verb", "l-participle", ..] => {
            let Some(cell) = crate::parse_l_participle_cell(feature) else {
                return (None, category);
            };
            api_metadata::l_participle_from_dictionary_metadata(metadata, cell)
        }
        ["verb", "participle", kind, "citation"] => {
            let Some(kind) = crate::parse_participle_kind(kind) else {
                return (None, category);
            };
            api_metadata::participle_from_dictionary_metadata(
                metadata,
                ParticipleCell {
                    kind,
                    adjective: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                    },
                },
            )
        }
        ["verb", "verbal-noun"] => api_metadata::verbal_noun_from_dictionary_metadata(
            metadata,
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
            },
        ),
        _ => return (None, category),
    };
    (form_set_prediction(result), category)
}

fn single_prediction(
    predicted: Result<old_church_slavonic_core::PredictedForm, old_church_slavonic::InflectionError>,
) -> Option<Vec<String>> {
    let predicted = predicted.ok()?;
    Some(vec![orthography::canonical_display(&predicted.text).ok()?])
}

fn form_set_prediction(
    result: Result<FormSet, old_church_slavonic::InflectionError>,
) -> Option<Vec<String>> {
    let set = result.ok()?;
    Some(set.variants().map(|variant| variant.text.clone()).collect())
}

fn form_code(form: AdjectiveForm) -> &'static str {
    match form {
        AdjectiveForm::Short => "short",
        AdjectiveForm::Long => "long",
    }
}

fn parse_noun_class(value: &str) -> Option<NounClass> {
    Some(match value {
        "o-m-hard" => NounClass::OMasculineHard,
        "o-n-hard" => NounClass::ONeuterHard,
        "a-hard" => NounClass::AHard,
        "jo-m-soft" => NounClass::JoMasculineSoft,
        "jo-n-soft" => NounClass::JoNeuterSoft,
        "ja-soft" => NounClass::JaSoft,
        "i-f" => NounClass::IFeminine,
        "i-m" => NounClass::IMasculine,
        "u-m" => NounClass::UMasculine,
        "n-m" => NounClass::NMasculine,
        "n-n" => NounClass::NNeuter,
        "nt-n" => NounClass::NtNeuter,
        "r-n" => NounClass::RStem,
        "s-n" => NounClass::SNeuter,
        "v-f" => NounClass::VFeminine,
        _ => return None,
    })
}

fn parse_gender(value: &str) -> Option<Gender> {
    Some(match value {
        "m" => Gender::Masculine,
        "f" => Gender::Feminine,
        "n" => Gender::Neuter,
        _ => return None,
    })
}

fn parse_animacy(value: &str) -> Option<Animacy> {
    Some(match value {
        "an" => Animacy::Animate,
        "in" => Animacy::Inanimate,
        _ => return None,
    })
}

fn render_markdown(
    by_pos: &BTreeMap<String, Counts>,
    divergent_categories: &BTreeMap<String, usize>,
    unsupported_categories: &BTreeMap<String, usize>,
    divergent_examples: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();
    out.push_str("# Rewrite derivability (phase 3 groundwork)\n\n");
    out.push_str(
        "How much of the extracted registry the pure rule kernel reproduces from\n\
         compact per-lexeme metadata alone (class codes, genders, verb\n\
         principal-part metadata), bypassing every stored surface form. A cell is\n\
         *derivable* when the rule prediction matches the stored variant list\n\
         exactly (Cyrillic text, variant order included); *divergent* when the\n\
         rules predict something else; *unsupported* when no rule prediction is\n\
         possible. Divergent and unsupported cells are the residue that must ship\n\
         as lookup tables.\n\n",
    );
    out.push_str("## Per part of speech\n\n");
    out.push_str(
        "| POS | cells | derivable | divergent | unsupported | derivable % | lexemes | fully derivable lexemes |\n",
    );
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|\n");
    let mut total = Counts::default();
    for (pos, counts) in by_pos {
        let _ = writeln!(
            out,
            "| {pos} | {} | {} | {} | {} | {:.2}% | {} | {} |",
            counts.cells,
            counts.derivable,
            counts.divergent,
            counts.unsupported,
            percent(counts.derivable, counts.cells),
            counts.lexemes,
            counts.lexemes_fully_derivable,
        );
        total.cells += counts.cells;
        total.derivable += counts.derivable;
        total.divergent += counts.divergent;
        total.divergent_primary_match += counts.divergent_primary_match;
        total.unsupported += counts.unsupported;
        total.lexemes += counts.lexemes;
        total.lexemes_fully_derivable += counts.lexemes_fully_derivable;
        total.residual_forms += counts.residual_forms;
        total.residual_form_bytes += counts.residual_form_bytes;
        total.residual_romanization_bytes += counts.residual_romanization_bytes;
    }
    let _ = writeln!(
        out,
        "| **total** | {} | {} | {} | {} | {:.2}% | {} | {} |",
        total.cells,
        total.derivable,
        total.divergent,
        total.unsupported,
        percent(total.derivable, total.cells),
        total.lexemes,
        total.lexemes_fully_derivable,
    );
    out.push_str("\n## Residual table estimate\n\n");
    let _ = writeln!(
        out,
        "- Residual cells (divergent + unsupported): {}",
        total.divergent + total.unsupported
    );
    let _ = writeln!(out, "- Residual surface variants: {}", total.residual_forms);
    let _ = writeln!(
        out,
        "- Divergent cells whose primary variant still matches the rules \
         (the table only adds or reorders variants): {} of {}",
        total.divergent_primary_match, total.divergent
    );
    let _ = writeln!(
        out,
        "- Residual form text: {} UTF-8 bytes (romanization adds {} bytes)",
        total.residual_form_bytes, total.residual_romanization_bytes
    );
    let _ = writeln!(
        out,
        "- Lexemes needing zero table rows: {} of {} ({:.2}%)",
        total.lexemes_fully_derivable,
        total.lexemes,
        percent(total.lexemes_fully_derivable, total.lexemes),
    );
    out.push_str("\n## Largest divergent categories\n\n");
    render_categories(&mut out, divergent_categories);
    out.push_str("\n## Largest unsupported categories\n\n");
    render_categories(&mut out, unsupported_categories);
    out.push_str("\n## Example divergences (first per top category)\n\n");
    let mut ranked: Vec<(&String, &usize)> = divergent_categories.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (category, count) in ranked.iter().take(10) {
        if let Some(example) = divergent_examples.get(*category) {
            let _ = writeln!(out, "- {category} ({count}): {example}");
        }
    }
    out.push_str(
        "\n## Notes\n\n\
         - The dominant adjective divergence is the animate accusative:\n\
           the extracted tables keep the plain accusative in `acc:pl/du:m:an`\n\
           cells while the core rules apply genitive syncretism. That is one\n\
           systematic convention difference, not thousands of independent\n\
           irregularities.\n\
         - Noun divergences are mostly extra stored variants (e.g. dative\n\
           `-оу / -еви`), `ѥ/е`-style orthographic variant spellings, and the\n\
           `-инъ` singulative subclass whose plural drops the suffix\n\
           (not modelled as a class).\n\
         - The verb residue is dominated by missing metadata, not failing\n\
           rules: `verb_metadata.tsv` covers only a minority of verbs per\n\
           system (about 121 present analyses, 185 l-participle stems for 711\n\
           verbs), so most verb cells have no rule input at all.\n\
         - Verb predictions consume `verb_metadata.tsv` (stems and formation\n\
           codes with provenance); that metadata is itself compact per-lexeme\n\
           data the rewrite would keep, not a surface table.\n\
         - Closed classes (pron/num/det `decl:*` cells) are counted as\n\
           unsupported here because this harness does not wire the reviewed\n\
           closed-class kernels; the core crate already models the major\n\
           paradigms as reviewed tables keyed by identity.\n\
         - Comparisons are on canonical Cyrillic text only; romanization is\n\
           assumed to be regenerable by transliteration.\n",
    );
    out
}

fn render_categories(out: &mut String, categories: &BTreeMap<String, usize>) {
    let mut rows: Vec<(&str, usize)> = categories
        .iter()
        .map(|(key, count)| (key.as_str(), *count))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    out.push_str("| category | cells |\n|---|---:|\n");
    for (key, count) in rows.iter().take(15) {
        let _ = writeln!(out, "| {key} | {count} |");
    }
    let remainder: usize = rows.iter().skip(15).map(|(_, count)| count).sum();
    if remainder > 0 {
        let _ = writeln!(out, "| (other) | {remainder} |");
    }
}

fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 * 100.0 / whole as f64
    }
}
