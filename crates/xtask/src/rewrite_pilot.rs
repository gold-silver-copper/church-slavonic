//! Rewrite pilot slice (phase 3/4): generate the noun residue + metadata
//! tables consumed by the new `church-slavonic` facade crate, and replay the
//! full attested noun oracle through that crate.
//!
//! - `cargo xtask rewrite-derivability --emit-residue` writes
//!   `crates/church-slavonic/generated/noun_residue.rs`.
//! - `cargo xtask rewrite-pilot-accuracy` replays every attested noun cell
//!   from `data/extracted` through `church_slavonic::noun_variants`.
//!
//! Oracle convention (documented in the facade's lib.rs): the oracle is
//! defined per lexeme. Homograph lexemes sharing a lemma get deterministic
//! numeric-suffix keys (`lemma`, `lemma_2`, ...), ordered by a pure sort of
//! their emitted form inventories (see `homograph_keys`); each key's cells
//! are that lexeme's own variant lists, never a union across senses.

use church_slavonic::{
    NounMeta, VerbCell, VerbMeta, adjective_cell_code, cell_code, closed_cell_code,
    kernel_closed_variants, kernel_noun_variants, kernel_verb_variants, verb_cell_code,
};
use old_church_slavonic::advanced::metadata as api_metadata;
use old_church_slavonic_core::{
    AdjectiveClass, AdjectiveForm, Animacy, AoristFormation, Case, FiniteTense, FiniteVerbCell,
    Gender, ImperativeCell, ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy,
    LParticipleCell, Number, NumberRestriction, PartOfSpeech, ParticipleKind,
    PastActiveParticipleFormation, PastPassiveParticipleFormation, Person,
    PresentActiveParticipleFormation, PresentFormation, PresentPassiveParticipleFormation,
    VerbAspect, VerbClass,
};
use old_church_slavonic_extractor::extract::load_registry;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// One lemma's compact encoded metadata: (class, gender, animacy, restriction).
type MetaCodes = (u8, u8, u8, u8);

/// A lexeme's rank-deduped form inventory: cell code -> variant list.
type Inventory = BTreeMap<u8, Vec<String>>;

/// Deterministic homograph keying (documented in the facade's lib.rs,
/// mirroring `gold-silver-copper/english`): lexemes sharing a lemma are
/// ordered by a pure sort of their emitted form inventories — the sorted
/// `(cell code, variant list)` sequence compared lexicographically, which is
/// exactly `BTreeMap<u8, Vec<String>>`'s derived `Ord` — with the encoded
/// lexeme metadata as tie-break. The first lexeme keeps the bare lemma; the
/// rest get `lemma_2`, `lemma_3`, … No external lockfile: the assignment is
/// reproducible from the data alone, and two lexemes tying under this sort
/// are content-identical, so their relative order cannot change any table.
fn homograph_keys<M: Ord>(
    lemma: &str,
    mut entries: Vec<(Inventory, M, String)>,
) -> Vec<(String, Inventory, M, String)> {
    entries.sort_by(|left, right| {
        (&left.0, &left.1)
            .cmp(&(&right.0, &right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    entries
        .into_iter()
        .enumerate()
        .map(|(index, (inventory, meta, lexeme_id))| {
            let key = if index == 0 {
                lemma.to_string()
            } else {
                format!("{lemma}_{}", index + 1)
            };
            (key, inventory, meta, lexeme_id)
        })
        .collect()
}

struct NounOracle {
    /// lemma key (bare or numeric-suffixed) -> encoded metadata for exactly
    /// that lexeme.
    meta: BTreeMap<String, MetaCodes>,
    /// (lemma key, cell code) -> that lexeme's own variant list in rank order.
    cells: BTreeMap<(String, u8), Vec<String>>,
    /// Homograph groups: (bare lemma, assigned keys in sense order).
    homographs: Vec<(String, Vec<String>)>,
    /// Extracted lexeme id -> assigned lemma key (bare or numeric-suffixed).
    key_by_lexeme: BTreeMap<String, String>,
}

fn class_code(value: &str) -> u8 {
    match value {
        "o-m-hard" => 1,
        "o-n-hard" => 2,
        "a-hard" => 3,
        "jo-m-soft" => 4,
        "jo-n-soft" => 5,
        "ja-soft" => 6,
        "i-f" => 7,
        "i-m" => 8,
        "u-m" => 9,
        "n-m" => 10,
        "n-n" => 11,
        "nt-n" => 12,
        "r-n" => 13,
        "s-n" => 14,
        "v-f" => 15,
        _ => 0,
    }
}

fn gender_code(value: &str) -> u8 {
    match value {
        "m" => 1,
        "f" => 2,
        "n" => 3,
        _ => 0,
    }
}

fn animacy_code(value: &str) -> u8 {
    match value {
        "an" => 1,
        "in" => 2,
        _ => 0,
    }
}

fn restriction_code(value: &str) -> u8 {
    match value {
        "sg" => 1,
        "du" => 2,
        "pl" => 3,
        _ => 0,
    }
}

fn decode_meta(codes: MetaCodes) -> NounMeta {
    NounMeta {
        class: match codes.0 {
            1 => Some(old_church_slavonic_core::NounClass::OMasculineHard),
            2 => Some(old_church_slavonic_core::NounClass::ONeuterHard),
            3 => Some(old_church_slavonic_core::NounClass::AHard),
            4 => Some(old_church_slavonic_core::NounClass::JoMasculineSoft),
            5 => Some(old_church_slavonic_core::NounClass::JoNeuterSoft),
            6 => Some(old_church_slavonic_core::NounClass::JaSoft),
            7 => Some(old_church_slavonic_core::NounClass::IFeminine),
            8 => Some(old_church_slavonic_core::NounClass::IMasculine),
            9 => Some(old_church_slavonic_core::NounClass::UMasculine),
            10 => Some(old_church_slavonic_core::NounClass::NMasculine),
            11 => Some(old_church_slavonic_core::NounClass::NNeuter),
            12 => Some(old_church_slavonic_core::NounClass::NtNeuter),
            13 => Some(old_church_slavonic_core::NounClass::RStem),
            14 => Some(old_church_slavonic_core::NounClass::SNeuter),
            15 => Some(old_church_slavonic_core::NounClass::VFeminine),
            _ => None,
        },
        gender: match codes.1 {
            1 => Some(Gender::Masculine),
            2 => Some(Gender::Feminine),
            3 => Some(Gender::Neuter),
            _ => None,
        },
        animacy: match codes.2 {
            1 => Some(Animacy::Animate),
            2 => Some(Animacy::Inanimate),
            _ => None,
        },
        restriction: match codes.3 {
            1 => NumberRestriction::SingularOnly,
            2 => NumberRestriction::DualOnly,
            3 => NumberRestriction::PluralOnly,
            _ => NumberRestriction::All,
        },
    }
}

fn feature_cell_code(feature: &str) -> Option<u8> {
    let cell = crate::parse_noun_cell(feature)?;
    Some(cell_code(cell.case, cell.number))
}

fn cell_from_code(code: u8) -> (Case, Number) {
    let case = match code / 3 {
        0 => Case::Nominative,
        1 => Case::Genitive,
        2 => Case::Dative,
        3 => Case::Accusative,
        4 => Case::Instrumental,
        5 => Case::Locative,
        _ => match code / 3 {
            6 => Case::Vocative,
            _ => Case::Vocative,
        },
    };
    let number = match code % 3 {
        0 => Number::Singular,
        1 => Number::Dual,
        _ => Number::Plural,
    };
    (case, number)
}

/// Rank-stable dedupe of one cell's raw rows into its variant list.
fn dedupe_ranked(rows: &mut Vec<(u16, String)>) -> Vec<String> {
    rows.sort_by_key(|(rank, _)| *rank);
    let mut texts: Vec<String> = Vec::new();
    for (_, form) in rows.iter() {
        if !texts.contains(form) {
            texts.push(form.clone());
        }
    }
    texts
}

fn load_noun_oracle(root: &Path) -> Result<NounOracle, Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    // lexeme id -> (lemma, encoded metadata).
    let mut lexemes: BTreeMap<&str, (&str, MetaCodes)> = BTreeMap::new();
    for lexeme in &registry.lexemes {
        if lexeme.pos != "noun" {
            continue;
        }
        lexemes.insert(
            &lexeme.id,
            (
                &lexeme.lemma,
                (
                    class_code(&lexeme.class),
                    gender_code(&lexeme.gender),
                    animacy_code(&lexeme.animacy),
                    restriction_code(&lexeme.number_restriction),
                ),
            ),
        );
    }
    // Per-lexeme rows, rank-stable within a cell.
    let mut per_lexeme: BTreeMap<&str, BTreeMap<u8, Vec<(u16, String)>>> = BTreeMap::new();
    for row in &registry.forms {
        if !lexemes.contains_key(row.lexeme_id.as_str()) {
            continue;
        }
        let Some(code) = feature_cell_code(&row.feature) else {
            return Err(format!("unparsed noun feature {}", row.feature).into());
        };
        per_lexeme
            .entry(row.lexeme_id.as_str())
            .or_default()
            .entry(code)
            .or_default()
            .push((row.rank, row.form.clone()));
    }
    // Group per-lexeme inventories by lemma and assign deterministic keys.
    let mut groups: BTreeMap<&str, Vec<(Inventory, MetaCodes, String)>> = BTreeMap::new();
    for (id, (lemma, codes)) in &lexemes {
        let inventory: Inventory = per_lexeme
            .remove(id)
            .unwrap_or_default()
            .into_iter()
            .map(|(code, mut rows)| (code, dedupe_ranked(&mut rows)))
            .collect();
        groups
            .entry(lemma)
            .or_default()
            .push((inventory, *codes, (*id).to_string()));
    }
    let mut meta: BTreeMap<String, MetaCodes> = BTreeMap::new();
    let mut cells: BTreeMap<(String, u8), Vec<String>> = BTreeMap::new();
    let mut homographs: Vec<(String, Vec<String>)> = Vec::new();
    let mut key_by_lexeme: BTreeMap<String, String> = BTreeMap::new();
    for (lemma, entries) in groups {
        let keyed = homograph_keys(lemma, entries);
        if keyed.len() > 1 {
            homographs.push((
                lemma.to_string(),
                keyed.iter().map(|(key, _, _, _)| key.clone()).collect(),
            ));
        }
        for (key, inventory, codes, lexeme_id) in keyed {
            key_by_lexeme.insert(lexeme_id, key.clone());
            meta.insert(key.clone(), codes);
            for (code, variants) in inventory {
                cells.insert((key.clone(), code), variants);
            }
        }
    }
    Ok(NounOracle {
        meta,
        cells,
        homographs,
        key_by_lexeme,
    })
}

/// Adjective oracle at lemma granularity, with the animacy dimension
/// collapsed after verifying it is degenerate (see the facade's lib.rs).
struct AdjectiveOracle {
    /// lemma key (bare or numeric-suffixed) -> encoded class
    /// (1 hard, 2 soft, 0 unknown) for exactly that lexeme.
    meta: BTreeMap<String, u8>,
    /// (lemma key, adjective cell code) -> that lexeme's own variant list.
    cells: BTreeMap<(String, u8), Vec<String>>,
    /// Attested `(lexeme, feature)` cells keyed with the animacy dimension,
    /// comparatives included — the raw 78,432 figure.
    keyed_cells: usize,
    /// `adj:comparative:citation` cells excluded from the facade + accuracy
    /// denominator (unpredictable suffix-grade / suppletive lexical facts).
    comparative_cells: usize,
    /// Homograph groups: (bare lemma, assigned keys in sense order).
    homographs: Vec<(String, Vec<String>)>,
    /// Extracted lexeme id -> assigned lemma key (bare or numeric-suffixed).
    key_by_lexeme: BTreeMap<String, String>,
}

fn adjective_class_code(value: &str) -> u8 {
    match value {
        "adj-hard" => 1,
        "adj-soft" => 2,
        _ => 0,
    }
}

fn decode_adjective_class(code: u8) -> Option<AdjectiveClass> {
    match code {
        1 => Some(AdjectiveClass::Hard),
        2 => Some(AdjectiveClass::Soft),
        _ => None,
    }
}

/// Decode an adjective cell code back into its typed dimensions.
fn adjective_cell_from_code(code: u8) -> (AdjectiveForm, Case, Number, Gender) {
    let form = if code / 63 == 0 {
        AdjectiveForm::Short
    } else {
        AdjectiveForm::Long
    };
    let (case, number) = cell_from_code((code % 63) / 3);
    let gender = match code % 3 {
        0 => Gender::Masculine,
        1 => Gender::Feminine,
        _ => Gender::Neuter,
    };
    (form, case, number, gender)
}

fn load_adjective_oracle(root: &Path) -> Result<AdjectiveOracle, Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    // lexeme id -> (lemma, encoded class).
    let mut lexemes: BTreeMap<&str, (&str, u8)> = BTreeMap::new();
    for lexeme in &registry.lexemes {
        if lexeme.pos != "adj" {
            continue;
        }
        lexemes.insert(
            &lexeme.id,
            (&lexeme.lemma, adjective_class_code(&lexeme.class)),
        );
    }
    // Rows keyed with the animacy dimension still present, to prove it is
    // degenerate before collapsing it out of the facade key.
    type Ranked = Vec<(u16, String)>;
    let mut per_lexeme: BTreeMap<(String, u8, Animacy), Ranked> = BTreeMap::new();
    let mut keyed_cells: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    let mut comparative_cells: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    for row in &registry.forms {
        if !lexemes.contains_key(row.lexeme_id.as_str()) {
            continue;
        }
        keyed_cells.insert((row.lexeme_id.clone(), row.feature.clone()));
        if row.feature == "adj:comparative:citation" {
            comparative_cells.insert(row.lexeme_id.clone());
            continue;
        }
        let Some(cell) = crate::parse_adjective_cell(&row.feature) else {
            return Err(format!("unparsed adjective feature {}", row.feature).into());
        };
        let code = adjective_cell_code(cell.form, cell.case, cell.number, cell.gender);
        per_lexeme
            .entry((row.lexeme_id.clone(), code, cell.animacy))
            .or_default()
            .push((row.rank, row.form.clone()));
    }
    // Collapse animacy per lexeme, failing loudly if the dimension ever
    // carries information (the facade API is built on this degeneracy).
    let mut collapsed_lexeme: BTreeMap<String, Inventory> = BTreeMap::new();
    for ((lexeme_id, code, animacy), mut rows) in per_lexeme {
        let texts = dedupe_ranked(&mut rows);
        match collapsed_lexeme
            .entry(lexeme_id.clone())
            .or_default()
            .entry(code)
        {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(texts);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                if entry.get() != &texts {
                    return Err(format!(
                        "adjective animacy dimension is not degenerate: {lexeme_id} cell {code} \
                         ({animacy:?}) stores {:?} vs {:?}",
                        texts,
                        entry.get()
                    )
                    .into());
                }
            }
        }
    }
    // Group per-lexeme inventories by lemma and assign deterministic keys.
    let mut groups: BTreeMap<&str, Vec<(Inventory, u8, String)>> = BTreeMap::new();
    for (id, (lemma, class)) in &lexemes {
        let inventory = collapsed_lexeme.remove(*id).unwrap_or_default();
        groups
            .entry(lemma)
            .or_default()
            .push((inventory, *class, (*id).to_string()));
    }
    let mut meta: BTreeMap<String, u8> = BTreeMap::new();
    let mut cells: BTreeMap<(String, u8), Vec<String>> = BTreeMap::new();
    let mut homographs: Vec<(String, Vec<String>)> = Vec::new();
    let mut key_by_lexeme: BTreeMap<String, String> = BTreeMap::new();
    for (lemma, entries) in groups {
        let keyed = homograph_keys(lemma, entries);
        if keyed.len() > 1 {
            homographs.push((
                lemma.to_string(),
                keyed.iter().map(|(key, _, _, _)| key.clone()).collect(),
            ));
        }
        for (key, inventory, class, lexeme_id) in keyed {
            key_by_lexeme.insert(lexeme_id, key.clone());
            meta.insert(key.clone(), class);
            for (code, variants) in inventory {
                cells.insert((key.clone(), code), variants);
            }
        }
    }
    Ok(AdjectiveOracle {
        meta,
        cells,
        keyed_cells: keyed_cells.len(),
        comparative_cells: comparative_cells.len(),
        homographs,
        key_by_lexeme,
    })
}

fn emit_adjective_residue(root: &Path) -> Result<(), Box<dyn Error>> {
    let oracle = load_adjective_oracle(root)?;
    let mut residue: Vec<(&str, u8, &Vec<String>)> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        let class = decode_adjective_class(oracle.meta[lemma]);
        let (form, case, number, gender) = adjective_cell_from_code(*code);
        let predicted = church_slavonic::kernel_adjective_variants(
            church_slavonic::base_lemma(lemma),
            class,
            form,
            case,
            number,
            gender,
        );
        if predicted.as_deref() != Some(expected.as_slice()) {
            residue.push((lemma, *code, expected));
        }
    }
    let mut out = String::new();
    out.push_str(
        "// @generated by `cargo xtask rewrite-derivability --emit-residue`. Do not edit.\n\
         //\n\
         // ADJ_META: (lemma, class) codes (1 hard, 2 soft, 0 unknown) for every\n\
         // adjective lemma in data/extracted. ADJ_RESIDUE: (lemma, adjective cell\n\
         // code, variants) for exactly the attested positive-degree cells the rule\n\
         // kernel does not reproduce verbatim; the cell code collapses the oracle's\n\
         // degenerate animacy dimension (see church-slavonic/src/lib.rs). Comparative\n\
         // citations are excluded from the facade. Both slices are sorted for binary\n\
         // search.\n",
    );
    let _ = writeln!(
        out,
        "pub static ADJ_META: &[(&str, u8)] = &[ // {} lemmas",
        oracle.meta.len()
    );
    for (lemma, class) in &oracle.meta {
        let _ = writeln!(out, "    ({lemma:?}, {class}),");
    }
    out.push_str("];\n");
    let _ = writeln!(
        out,
        "pub static ADJ_RESIDUE: &[(&str, u8, &[&str])] = &[ // {} cells",
        residue.len()
    );
    for (lemma, code, variants) in &residue {
        let _ = write!(out, "    ({lemma:?}, {code}, &[");
        for (index, variant) in variants.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{variant:?}");
        }
        out.push_str("]),\n");
    }
    out.push_str("];\n");
    let path = root.join("crates/church-slavonic/generated/adjective_residue.rs");
    fs::write(&path, &out)?;
    println!(
        "wrote {} ({} bytes): {} metadata rows, {} residue cells (of {} per-lexeme cells; \
         {} comparative cells excluded; {} homograph groups)",
        path.display(),
        out.len(),
        oracle.meta.len(),
        residue.len(),
        oracle.cells.len(),
        oracle.comparative_cells,
        oracle.homographs.len(),
    );
    Ok(())
}

/// Compact per-lemma verb principal-part codes, owned by the emitter before
/// being written as `crate::VerbMeta` literals (see the facade's field code
/// tables on `VerbMeta`).
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct VerbMetaCodes {
    aspect: u8,
    present: Vec<(String, u8, String, String, u8)>,
    imperfect: Vec<(String, u8, u8)>,
    aorist: Vec<(String, String, u8)>,
    imperative: Vec<(String, u8)>,
    l_participle: Vec<String>,
    present_active_participle: Vec<(String, u8)>,
    present_passive_participle: Vec<(String, u8)>,
    past_active_participle: Vec<(String, u8)>,
    past_passive_participle: Vec<(String, u8)>,
}

fn verb_class_code(value: VerbClass) -> u8 {
    match value {
        VerbClass::IA1 => 1,
        VerbClass::IA2 => 2,
        VerbClass::II1 => 3,
        VerbClass::II2 => 4,
        VerbClass::II3 => 5,
        VerbClass::Root => 6,
        VerbClass::Irregular => 7,
    }
}

fn verb_aspect_code(value: VerbAspect) -> u8 {
    match value {
        VerbAspect::Perfective => 1,
        VerbAspect::Imperfective => 2,
        VerbAspect::Biaspectual => 3,
    }
}

fn present_formation_code(value: PresentFormation) -> u8 {
    match value {
        PresentFormation::IotatedE => 1,
        PresentFormation::HardI => 2,
    }
}

fn imperfect_formation_code(value: ImperfectFormation) -> u8 {
    match value {
        ImperfectFormation::A => 1,
        ImperfectFormation::YatA => 2,
        ImperfectFormation::PalatalizedA => 3,
        ImperfectFormation::PresentA => 4,
        ImperfectFormation::PresentYatA => 5,
    }
}

fn imperfect_policy_code(value: ImperfectVariantPolicy) -> u8 {
    match value {
        ImperfectVariantPolicy::UncontractedOnly => 1,
        ImperfectVariantPolicy::ContractedOnly => 2,
        ImperfectVariantPolicy::IotatedOnly => 3,
    }
}

fn aorist_formation_code(value: AoristFormation) -> u8 {
    match value {
        AoristFormation::Asigmatic => 1,
        AoristFormation::SigmaticPrimary => 2,
        AoristFormation::SigmaticSecondary => 3,
        AoristFormation::SigmaticVowel => 4,
        AoristFormation::New => 5,
    }
}

fn imperative_formation_code(value: ImperativeFormation) -> u8 {
    match value {
        ImperativeFormation::ISeries => 1,
        ImperativeFormation::YatSeries => 2,
    }
}

fn present_active_participle_code(value: PresentActiveParticipleFormation) -> u8 {
    match value {
        PresentActiveParticipleFormation::YushtHard => 1,
        PresentActiveParticipleFormation::YushtSoft => 2,
        PresentActiveParticipleFormation::YeshtSoft => 3,
        PresentActiveParticipleFormation::MixedYushtSoft => 4,
        PresentActiveParticipleFormation::IotatedYushtSoft => 5,
    }
}

fn present_passive_participle_code(value: PresentPassiveParticipleFormation) -> u8 {
    match value {
        PresentPassiveParticipleFormation::Im => 1,
        PresentPassiveParticipleFormation::Em => 2,
        PresentPassiveParticipleFormation::IotatedEm => 3,
        PresentPassiveParticipleFormation::Om => 4,
    }
}

fn past_active_participle_code(value: PastActiveParticipleFormation) -> u8 {
    match value {
        PastActiveParticipleFormation::Ush => 1,
        PastActiveParticipleFormation::Ish => 2,
        PastActiveParticipleFormation::IshAfterGlide => 3,
        PastActiveParticipleFormation::VushAfterJDeletion => 4,
        PastActiveParticipleFormation::VushAfterOvToU => 5,
        PastActiveParticipleFormation::Vush => 6,
    }
}

fn past_passive_participle_code(value: PastPassiveParticipleFormation) -> u8 {
    match value {
        PastPassiveParticipleFormation::T => 1,
        PastPassiveParticipleFormation::N => 2,
        PastPassiveParticipleFormation::En => 3,
    }
}

fn encode_verb_metadata(metadata: &api_metadata::DictionaryVerbMetadata) -> VerbMetaCodes {
    VerbMetaCodes {
        aspect: metadata
            .aspect
            .as_ref()
            .map_or(0, |aspect| verb_aspect_code(aspect.value)),
        present: metadata
            .present
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    verb_class_code(analysis.class.value),
                    analysis
                        .first_singular_stem
                        .as_ref()
                        .map_or_else(String::new, |stem| stem.value.clone()),
                    analysis
                        .third_plural_stem
                        .as_ref()
                        .map_or_else(String::new, |stem| stem.value.clone()),
                    analysis
                        .formation
                        .as_ref()
                        .map_or(0, |formation| present_formation_code(formation.value)),
                )
            })
            .collect(),
        imperfect: metadata
            .imperfect
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    imperfect_formation_code(analysis.formation.value),
                    imperfect_policy_code(analysis.variant_policy.value),
                )
            })
            .collect(),
        aorist: metadata
            .aorist
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    analysis
                        .second_third_singular
                        .as_ref()
                        .map_or_else(String::new, |part| part.value.clone()),
                    aorist_formation_code(analysis.formation.value),
                )
            })
            .collect(),
        imperative: metadata
            .imperative
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    imperative_formation_code(analysis.formation.value),
                )
            })
            .collect(),
        l_participle: metadata
            .l_participle
            .iter()
            .map(|analysis| analysis.stem.value.clone())
            .collect(),
        present_active_participle: metadata
            .present_active_participle
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    present_active_participle_code(analysis.formation.value),
                )
            })
            .collect(),
        present_passive_participle: metadata
            .present_passive_participle
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    present_passive_participle_code(analysis.formation.value),
                )
            })
            .collect(),
        past_active_participle: metadata
            .past_active_participle
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    past_active_participle_code(analysis.formation.value),
                )
            })
            .collect(),
        past_passive_participle: metadata
            .past_passive_participle
            .iter()
            .map(|analysis| {
                (
                    analysis.stem.value.clone(),
                    past_passive_participle_code(analysis.formation.value),
                )
            })
            .collect(),
    }
}

/// Run `body` with a borrowed `VerbMeta` view over the owned code rows —
/// exactly the record the facade decodes from the generated table.
fn with_verb_meta_view<R>(codes: &VerbMetaCodes, body: impl FnOnce(&VerbMeta<'_>) -> R) -> R {
    let present: Vec<(&str, u8, &str, &str, u8)> = codes
        .present
        .iter()
        .map(|(stem, class, first, third, formation)| {
            (
                stem.as_str(),
                *class,
                first.as_str(),
                third.as_str(),
                *formation,
            )
        })
        .collect();
    let imperfect: Vec<(&str, u8, u8)> = codes
        .imperfect
        .iter()
        .map(|(stem, formation, policy)| (stem.as_str(), *formation, *policy))
        .collect();
    let aorist: Vec<(&str, &str, u8)> = codes
        .aorist
        .iter()
        .map(|(stem, singular, formation)| (stem.as_str(), singular.as_str(), *formation))
        .collect();
    fn pair(rows: &[(String, u8)]) -> Vec<(&str, u8)> {
        rows.iter()
            .map(|(stem, formation)| (stem.as_str(), *formation))
            .collect()
    }
    let imperative = pair(&codes.imperative);
    let l_participle: Vec<&str> = codes.l_participle.iter().map(String::as_str).collect();
    let present_active_participle = pair(&codes.present_active_participle);
    let present_passive_participle = pair(&codes.present_passive_participle);
    let past_active_participle = pair(&codes.past_active_participle);
    let past_passive_participle = pair(&codes.past_passive_participle);
    let meta = VerbMeta {
        aspect: codes.aspect,
        present: &present,
        imperfect: &imperfect,
        aorist: &aorist,
        imperative: &imperative,
        l_participle: &l_participle,
        present_active_participle: &present_active_participle,
        present_passive_participle: &present_passive_participle,
        past_active_participle: &past_active_participle,
        past_passive_participle: &past_passive_participle,
    };
    body(&meta)
}

struct VerbOracle {
    /// lemma key (bare or numeric-suffixed) -> that lexeme's own encoded
    /// principal-part metadata.
    meta: BTreeMap<String, VerbMetaCodes>,
    /// (lemma key, verb cell code) -> that lexeme's own variant list.
    cells: BTreeMap<(String, u8), Vec<String>>,
    /// Homograph groups: (bare lemma, assigned keys in sense order).
    homographs: Vec<(String, Vec<String>)>,
    /// Extracted lexeme id -> assigned lemma key (bare or numeric-suffixed).
    key_by_lexeme: BTreeMap<String, String>,
}

fn parse_verb_cell(feature: &str) -> Option<VerbCell> {
    match feature {
        "verb:infinitive" => return Some(VerbCell::Infinitive),
        "verb:supine" => return Some(VerbCell::Supine),
        "verb:verbal-noun" => return Some(VerbCell::VerbalNoun),
        _ => {}
    }
    if let Some(cell) = crate::parse_finite_verb_cell(feature) {
        return Some(VerbCell::Finite(cell));
    }
    if let Some(cell) = crate::parse_imperative_cell(feature) {
        return Some(VerbCell::Imperative(cell));
    }
    if let Some(cell) = crate::parse_l_participle_cell(feature) {
        return Some(VerbCell::LParticiple(cell));
    }
    let parts: Vec<&str> = feature.split(':').collect();
    if let ["verb", "participle", kind, "citation"] = parts.as_slice() {
        return crate::parse_participle_kind(kind).map(VerbCell::ParticipleCitation);
    }
    None
}

/// Decode a verb cell code back into its typed cell (inverse of the facade's
/// `verb_cell_code`).
fn verb_cell_from_code(code: u8) -> VerbCell {
    let person = |index: u8| match index {
        0 => Person::First,
        1 => Person::Second,
        _ => Person::Third,
    };
    let number = |index: u8| match index {
        0 => Number::Singular,
        1 => Number::Dual,
        _ => Number::Plural,
    };
    match code {
        0..=26 => VerbCell::Finite(FiniteVerbCell {
            tense: match code / 9 {
                0 => FiniteTense::Present,
                1 => FiniteTense::Imperfect,
                _ => FiniteTense::Aorist,
            },
            person: person((code % 9) / 3),
            number: number(code % 3),
        }),
        27..=35 => VerbCell::Imperative(ImperativeCell {
            person: person((code - 27) / 3),
            number: number(code % 3),
        }),
        36..=44 => VerbCell::LParticiple(LParticipleCell {
            gender: match (code - 36) / 3 {
                0 => Gender::Masculine,
                1 => Gender::Feminine,
                _ => Gender::Neuter,
            },
            number: number(code % 3),
        }),
        45 => VerbCell::Infinitive,
        46 => VerbCell::Supine,
        47 => VerbCell::VerbalNoun,
        _ => VerbCell::ParticipleCitation(match code - 48 {
            0 => ParticipleKind::PresentActive,
            1 => ParticipleKind::PresentPassive,
            2 => ParticipleKind::PastActive,
            _ => ParticipleKind::PastPassive,
        }),
    }
}

fn load_verb_oracle(root: &Path) -> Result<VerbOracle, Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    // lexeme id -> (lemma, that lexeme's own encoded metadata).
    let mut lexemes: BTreeMap<&str, (&str, VerbMetaCodes)> = BTreeMap::new();
    for lexeme in &registry.lexemes {
        if lexeme.pos != "verb" {
            continue;
        }
        let codes = api_metadata::verb_metadata_by_id(&lexeme.id)
            .map(|metadata| encode_verb_metadata(&metadata))
            .unwrap_or_default();
        lexemes.insert(&lexeme.id, (&lexeme.lemma, codes));
    }
    let mut per_lexeme: BTreeMap<&str, BTreeMap<u8, Vec<(u16, String)>>> = BTreeMap::new();
    for row in &registry.forms {
        if !lexemes.contains_key(row.lexeme_id.as_str()) {
            continue;
        }
        let Some(cell) = parse_verb_cell(&row.feature) else {
            return Err(format!("unparsed verb feature {}", row.feature).into());
        };
        let code = verb_cell_code(cell);
        per_lexeme
            .entry(row.lexeme_id.as_str())
            .or_default()
            .entry(code)
            .or_default()
            .push((row.rank, row.form.clone()));
    }
    // Group per-lexeme inventories by lemma and assign deterministic keys.
    let mut groups: BTreeMap<&str, Vec<(Inventory, VerbMetaCodes, String)>> = BTreeMap::new();
    for (id, (lemma, codes)) in &lexemes {
        let inventory: Inventory = per_lexeme
            .remove(id)
            .unwrap_or_default()
            .into_iter()
            .map(|(code, mut rows)| (code, dedupe_ranked(&mut rows)))
            .collect();
        groups
            .entry(lemma)
            .or_default()
            .push((inventory, codes.clone(), (*id).to_string()));
    }
    let mut meta: BTreeMap<String, VerbMetaCodes> = BTreeMap::new();
    let mut cells: BTreeMap<(String, u8), Vec<String>> = BTreeMap::new();
    let mut homographs: Vec<(String, Vec<String>)> = Vec::new();
    let mut key_by_lexeme: BTreeMap<String, String> = BTreeMap::new();
    for (lemma, entries) in groups {
        let keyed = homograph_keys(lemma, entries);
        if keyed.len() > 1 {
            homographs.push((
                lemma.to_string(),
                keyed.iter().map(|(key, _, _, _)| key.clone()).collect(),
            ));
        }
        for (key, inventory, codes, lexeme_id) in keyed {
            key_by_lexeme.insert(lexeme_id, key.clone());
            meta.insert(key.clone(), codes);
            for (code, variants) in inventory {
                cells.insert((key.clone(), code), variants);
            }
        }
    }
    Ok(VerbOracle {
        meta,
        cells,
        homographs,
        key_by_lexeme,
    })
}

/// ---- Principal-part metadata synthesis from the attested oracle ----
///
/// For each verb lemma whose compact metadata leaves a whole inflectional
/// system empty (no analyses), enumerate candidate analyses the core kernel
/// supports — candidate stems are derived from the attested surface forms
/// themselves (prefix stripping bounded by `SYNTH_MAX_STRIP` characters,
/// plus the exact attested 2/3sg aorist forms for the syncretic sigmatic
/// principal part) crossed with every formation code the system admits.
/// A candidate analysis is kept only when replaying it through
/// `kernel_verb_variants` yields, for every attested cell of the system, a
/// text that appears in that cell's stored variant list. A bounded
/// depth-first search (`SYNTH_MAX_DEPTH` analyses) then looks for an ordered
/// analysis list whose merge (analysis order, duplicates dropped — the
/// facade's `merge_analyses` semantics) reproduces every attested variant
/// list exactly. Candidates are sorted, so the first solution found is the
/// lexicographically smallest encoding: the whole procedure is
/// deterministic. Systems with no fitting candidate keep their residue rows;
/// the residue loop re-verifies every cell afterwards, so the 100% gates
/// hold by construction.
///
/// The search never touches a system that already has analyses, and the
/// present system is the only one whose class code leaks into other systems
/// (via the facade's `base_verb_lexeme`) — the core consults `lexeme.class`
/// only inside `present()`, so filling one system cannot regress another.
const SYNTH_MAX_STRIP: usize = 8;
const SYNTH_MAX_DEPTH: usize = 3;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum SynthAnalysis {
    /// (stem, class, first-singular stem, third-plural stem, formation)
    Present(String, u8, String, String, u8),
    /// (stem, formation, variant policy)
    Imperfect(String, u8, u8),
    /// (stem, 2/3sg principal part, formation)
    Aorist(String, String, u8),
    Imperative(String, u8),
    LParticiple(String),
    PresentActiveParticiple(String, u8),
    PresentPassiveParticiple(String, u8),
    PastActiveParticiple(String, u8),
    PastPassiveParticiple(String, u8),
}

fn synth_install(codes: &mut VerbMetaCodes, analysis: &SynthAnalysis) {
    match analysis {
        SynthAnalysis::Present(stem, class, first, third, formation) => codes.present.push((
            stem.clone(),
            *class,
            first.clone(),
            third.clone(),
            *formation,
        )),
        SynthAnalysis::Imperfect(stem, formation, policy) => {
            codes.imperfect.push((stem.clone(), *formation, *policy));
        }
        SynthAnalysis::Aorist(stem, singular, formation) => {
            codes
                .aorist
                .push((stem.clone(), singular.clone(), *formation));
        }
        SynthAnalysis::Imperative(stem, formation) => {
            codes.imperative.push((stem.clone(), *formation));
        }
        SynthAnalysis::LParticiple(stem) => codes.l_participle.push(stem.clone()),
        SynthAnalysis::PresentActiveParticiple(stem, formation) => {
            codes
                .present_active_participle
                .push((stem.clone(), *formation));
        }
        SynthAnalysis::PresentPassiveParticiple(stem, formation) => {
            codes
                .present_passive_participle
                .push((stem.clone(), *formation));
        }
        SynthAnalysis::PastActiveParticiple(stem, formation) => {
            codes
                .past_active_participle
                .push((stem.clone(), *formation));
        }
        SynthAnalysis::PastPassiveParticiple(stem, formation) => {
            codes
                .past_passive_participle
                .push((stem.clone(), *formation));
        }
    }
}

/// Candidate stems: every char-boundary prefix of every attested variant,
/// stripping at most `SYNTH_MAX_STRIP` characters (endings, suffix bundles
/// and stem mutations are all shorter than that), minimum one character.
fn synth_stem_pool(cells: &[(u8, &Vec<String>)]) -> Vec<String> {
    let mut pool: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (_, variants) in cells {
        for form in variants.iter() {
            let chars: Vec<char> = form.chars().collect();
            let max_strip = SYNTH_MAX_STRIP.min(chars.len().saturating_sub(1));
            for strip in 0..=max_strip {
                pool.insert(chars[..chars.len() - strip].iter().collect());
            }
        }
    }
    pool.into_iter().collect()
}

/// One analysis's single-text prediction for one cell, replayed through the
/// exact kernel path the facade uses (identity kernels first, then the
/// metadata generators over a probe record holding just this analysis).
fn synth_predict(
    lemma: &str,
    base: &VerbMetaCodes,
    analysis: &SynthAnalysis,
    code: u8,
) -> Option<String> {
    let mut probe = base.clone();
    synth_install(&mut probe, analysis);
    let texts = with_verb_meta_view(&probe, |meta| {
        kernel_verb_variants(lemma, meta, verb_cell_from_code(code))
    })?;
    if texts.len() == 1 {
        texts.into_iter().next()
    } else {
        None
    }
}

/// Validate a candidate over every attested cell of the system: each
/// prediction must exist and appear somewhere in that cell's variant list.
fn synth_candidate(
    lemma: &str,
    base: &VerbMetaCodes,
    analysis: SynthAnalysis,
    cells: &[(u8, &Vec<String>)],
) -> Option<(SynthAnalysis, Vec<String>)> {
    let mut predictions = Vec::with_capacity(cells.len());
    for (code, variants) in cells {
        let text = synth_predict(lemma, base, &analysis, *code)?;
        if !variants.contains(&text) {
            return None;
        }
        predictions.push(text);
    }
    Some((analysis, predictions))
}

/// Bounded DFS for an ordered candidate list whose merge reproduces every
/// attested variant list exactly. Candidates must be pre-sorted; the first
/// solution found is returned, making the choice deterministic.
fn synth_search(
    cells: &[(u8, &Vec<String>)],
    candidates: &[(SynthAnalysis, Vec<String>)],
) -> Option<Vec<SynthAnalysis>> {
    fn dfs(
        targets: &[&Vec<String>],
        candidates: &[(SynthAnalysis, Vec<String>)],
        consumed: &mut Vec<usize>,
        chosen: &mut Vec<SynthAnalysis>,
    ) -> bool {
        if targets
            .iter()
            .zip(consumed.iter())
            .all(|(list, used)| list.len() == *used)
        {
            return true;
        }
        if chosen.len() >= SYNTH_MAX_DEPTH {
            return false;
        }
        'candidate: for (analysis, predictions) in candidates {
            let mut next = consumed.clone();
            let mut advanced = false;
            for (index, text) in predictions.iter().enumerate() {
                let list = targets[index];
                if next[index] < list.len() && &list[next[index]] == text {
                    next[index] += 1;
                    advanced = true;
                } else if !list[..next[index]].contains(text) {
                    continue 'candidate;
                }
            }
            if !advanced {
                continue;
            }
            chosen.push(analysis.clone());
            let saved = std::mem::replace(consumed, next);
            if dfs(targets, candidates, consumed, chosen) {
                return true;
            }
            *consumed = saved;
            chosen.pop();
        }
        false
    }
    let targets: Vec<&Vec<String>> = cells.iter().map(|(_, list)| *list).collect();
    let mut consumed = vec![0usize; cells.len()];
    let mut chosen = Vec::new();
    dfs(&targets, candidates, &mut consumed, &mut chosen).then_some(chosen)
}

/// Enumerate, validate, sort and search one system's candidate analyses.
fn synth_system(
    lemma: &str,
    base: &VerbMetaCodes,
    cells: &[(u8, &Vec<String>)],
    enumerate: impl Fn(&str) -> Vec<SynthAnalysis>,
) -> Option<Vec<SynthAnalysis>> {
    let mut candidates: Vec<(SynthAnalysis, Vec<String>)> = Vec::new();
    for stem in synth_stem_pool(cells) {
        for analysis in enumerate(&stem) {
            if let Some(candidate) = synth_candidate(lemma, base, analysis, cells) {
                candidates.push(candidate);
            }
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates.dedup_by(|left, right| left.0 == right.0);
    synth_search(cells, &candidates)
}

/// The present system's candidates carry optional 1sg/3pl allomorph stems.
/// The default-stem candidate must already fit every non-allomorph cell;
/// allomorph stems are then solved independently (each affects exactly one
/// cell), keeping the enumeration small.
fn synth_present_system(
    lemma: &str,
    base: &VerbMetaCodes,
    cells: &[(u8, &Vec<String>)],
) -> Option<Vec<SynthAnalysis>> {
    // (class, formation) pairs the core's `present()` accepts.
    const COMBOS: [(u8, u8); 10] = [
        (1, 0),
        (1, 1),
        (2, 0),
        (2, 1),
        (3, 0),
        (3, 2),
        (4, 0),
        (4, 2),
        (5, 0),
        (5, 2),
    ];
    let first_singular: Vec<(u8, &Vec<String>)> = cells
        .iter()
        .filter(|(code, _)| *code == 0)
        .copied()
        .collect();
    let third_plural: Vec<(u8, &Vec<String>)> = cells
        .iter()
        .filter(|(code, _)| *code == 8)
        .copied()
        .collect();
    let middle: Vec<(u8, &Vec<String>)> = cells
        .iter()
        .filter(|(code, _)| *code != 0 && *code != 8)
        .copied()
        .collect();
    let allomorph_pool = |edge: &[(u8, &Vec<String>)]| -> Vec<String> {
        let mut pool = synth_stem_pool(edge);
        pool.insert(0, String::new());
        pool.dedup();
        pool
    };
    let first_pool = allomorph_pool(&first_singular);
    let third_pool = allomorph_pool(&third_plural);
    let mut candidates: Vec<(SynthAnalysis, Vec<String>)> = Vec::new();
    for stem in synth_stem_pool(cells) {
        for (class, formation) in COMBOS {
            let plain = SynthAnalysis::Present(
                stem.clone(),
                class,
                String::new(),
                String::new(),
                formation,
            );
            // The default stem must serve every middle cell on its own.
            if synth_candidate(lemma, base, plain.clone(), &middle).is_none() {
                continue;
            }
            let edge_options = |edge: &[(u8, &Vec<String>)], pool: &[String], first: bool| {
                if edge.is_empty() {
                    return vec![String::new()];
                }
                pool.iter()
                    .filter(|allomorph| {
                        let analysis = if first {
                            SynthAnalysis::Present(
                                stem.clone(),
                                class,
                                (*allomorph).clone(),
                                String::new(),
                                formation,
                            )
                        } else {
                            SynthAnalysis::Present(
                                stem.clone(),
                                class,
                                String::new(),
                                (*allomorph).clone(),
                                formation,
                            )
                        };
                        synth_candidate(lemma, base, analysis, edge).is_some()
                    })
                    .cloned()
                    .collect::<Vec<String>>()
            };
            let first_options = edge_options(&first_singular, &first_pool, true);
            let third_options = edge_options(&third_plural, &third_pool, false);
            for first_stem in &first_options {
                for third_stem in &third_options {
                    let analysis = SynthAnalysis::Present(
                        stem.clone(),
                        class,
                        first_stem.clone(),
                        third_stem.clone(),
                        formation,
                    );
                    if let Some(candidate) = synth_candidate(lemma, base, analysis, cells) {
                        candidates.push(candidate);
                    }
                }
            }
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates.dedup_by(|left, right| left.0 == right.0);
    synth_search(cells, &candidates)
}

/// Fill empty metadata systems by inference from the attested oracle.
/// Returns (lemmas gaining synthesized analyses, systems filled).
fn synthesize_verb_metadata(oracle: &mut VerbOracle) -> (usize, usize) {
    let mut cells_by_lemma: BTreeMap<&String, Vec<(u8, &Vec<String>)>> = BTreeMap::new();
    for ((lemma, code), variants) in &oracle.cells {
        cells_by_lemma
            .entry(lemma)
            .or_default()
            .push((*code, variants));
    }
    let mut lemmas_touched = 0usize;
    let mut systems_filled = 0usize;
    let mut synthesized: Vec<(String, VerbMetaCodes)> = Vec::new();
    for (lemma, cells) in &cells_by_lemma {
        // Kernel derivation always sees the surface lemma, never the
        // numeric homograph suffix.
        let kernel_lemma = church_slavonic::base_lemma(lemma);
        let base = oracle.meta[lemma.as_str()].clone();
        // Cells the metadata path will actually govern: the kernel currently
        // returns nothing for them (identity-served cells are unaffected by
        // metadata and are excluded), and the cell is one the metadata
        // generators can express at all.
        let eligible = |codes: &VerbMetaCodes, code: u8| -> bool {
            if let VerbCell::Imperative(cell) = verb_cell_from_code(code) {
                if !cell.is_supported() {
                    return false;
                }
            }
            with_verb_meta_view(codes, |meta| {
                kernel_verb_variants(kernel_lemma, meta, verb_cell_from_code(code))
            })
            .is_none()
        };
        // (system id, cell-code filter, whether the analysis list is empty)
        let mut updated = base.clone();
        let mut touched = false;
        let systems: [(bool, fn(u8) -> bool); 9] = [
            (base.present.is_empty(), |code| code <= 8),
            (base.imperfect.is_empty(), |code| (9..=17).contains(&code)),
            (base.aorist.is_empty(), |code| (18..=26).contains(&code)),
            (base.imperative.is_empty(), |code| (27..=35).contains(&code)),
            (base.l_participle.is_empty(), |code| {
                (36..=44).contains(&code)
            }),
            (base.present_active_participle.is_empty(), |code| code == 48),
            (base.present_passive_participle.is_empty(), |code| {
                code == 49
            }),
            (base.past_active_participle.is_empty(), |code| code == 50),
            (base.past_passive_participle.is_empty(), |code| {
                code == 47 || code == 51
            }),
        ];
        for (system_index, (empty, filter)) in systems.iter().enumerate() {
            if !empty {
                continue;
            }
            let system_cells: Vec<(u8, &Vec<String>)> = cells
                .iter()
                .filter(|(code, _)| filter(*code) && eligible(&updated, *code))
                .copied()
                .collect();
            if system_cells.is_empty() {
                continue;
            }
            let solution = match system_index {
                0 => synth_present_system(kernel_lemma, &updated, &system_cells),
                1 => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    let mut analyses = Vec::new();
                    for formation in 1..=5u8 {
                        for policy in 1..=3u8 {
                            analyses.push(SynthAnalysis::Imperfect(
                                stem.to_string(),
                                formation,
                                policy,
                            ));
                        }
                    }
                    analyses
                }),
                2 => {
                    // Syncretic 2/3sg sigmatic principal parts are the exact
                    // attested 2sg/3sg forms, not stripped prefixes.
                    let mut syncretic: Vec<String> = vec![String::new()];
                    for (code, variants) in &system_cells {
                        if *code == 21 || *code == 24 {
                            for form in variants.iter() {
                                if !syncretic.contains(form) {
                                    syncretic.push(form.clone());
                                }
                            }
                        }
                    }
                    synth_system(kernel_lemma, &updated, &system_cells, move |stem| {
                        let mut analyses = Vec::new();
                        for formation in [1u8, 5] {
                            analyses.push(SynthAnalysis::Aorist(
                                stem.to_string(),
                                String::new(),
                                formation,
                            ));
                        }
                        for formation in [2u8, 3, 4] {
                            for singular in &syncretic {
                                analyses.push(SynthAnalysis::Aorist(
                                    stem.to_string(),
                                    singular.clone(),
                                    formation,
                                ));
                            }
                        }
                        analyses
                    })
                }
                3 => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    (1..=2u8)
                        .map(|formation| SynthAnalysis::Imperative(stem.to_string(), formation))
                        .collect()
                }),
                4 => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    vec![SynthAnalysis::LParticiple(stem.to_string())]
                }),
                5 => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    (1..=5u8)
                        .map(|formation| {
                            SynthAnalysis::PresentActiveParticiple(stem.to_string(), formation)
                        })
                        .collect()
                }),
                6 => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    (1..=4u8)
                        .map(|formation| {
                            SynthAnalysis::PresentPassiveParticiple(stem.to_string(), formation)
                        })
                        .collect()
                }),
                7 => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    (1..=6u8)
                        .map(|formation| {
                            SynthAnalysis::PastActiveParticiple(stem.to_string(), formation)
                        })
                        .collect()
                }),
                _ => synth_system(kernel_lemma, &updated, &system_cells, |stem| {
                    (1..=3u8)
                        .map(|formation| {
                            SynthAnalysis::PastPassiveParticiple(stem.to_string(), formation)
                        })
                        .collect()
                }),
            };
            if let Some(analyses) = solution {
                for analysis in &analyses {
                    synth_install(&mut updated, analysis);
                }
                systems_filled += 1;
                touched = true;
            }
        }
        if touched {
            lemmas_touched += 1;
            synthesized.push(((*lemma).clone(), updated));
        }
    }
    for (lemma, codes) in synthesized {
        oracle.meta.insert(lemma, codes);
    }
    (lemmas_touched, systems_filled)
}

fn write_str_pair_slice(out: &mut String, rows: &[(String, u8)]) {
    out.push_str("&[");
    for (index, (stem, code)) in rows.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "({stem:?}, {code})");
    }
    out.push(']');
}

fn verb_residue_cells(oracle: &VerbOracle) -> Vec<(&str, u8, &Vec<String>)> {
    let mut residue: Vec<(&str, u8, &Vec<String>)> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        let predicted = with_verb_meta_view(&oracle.meta[lemma], |meta| {
            kernel_verb_variants(
                church_slavonic::base_lemma(lemma),
                meta,
                verb_cell_from_code(*code),
            )
        });
        if predicted.as_deref() != Some(expected.as_slice()) {
            residue.push((lemma, *code, expected));
        }
    }
    residue
}

fn emit_verb_residue(root: &Path) -> Result<(), Box<dyn Error>> {
    let mut oracle = load_verb_oracle(root)?;
    let residue_before = verb_residue_cells(&oracle).len();
    let fully_rules = |oracle: &VerbOracle| {
        let residue_lemmas: std::collections::BTreeSet<&str> = verb_residue_cells(oracle)
            .iter()
            .map(|(lemma, _, _)| *lemma)
            .collect();
        oracle.meta.len() - residue_lemmas.len()
    };
    let fully_before = fully_rules(&oracle);
    let (synth_lemmas, synth_systems) = synthesize_verb_metadata(&mut oracle);
    let residue = verb_residue_cells(&oracle);
    let fully_after = fully_rules(&oracle);
    println!(
        "verb metadata synthesis: {synth_lemmas} lemmas gained {synth_systems} synthesized \
         systems; residue cells {residue_before} -> {}; fully rules-backed lemmas \
         {fully_before} -> {fully_after}",
        residue.len(),
    );
    let mut out = String::new();
    out.push_str(
        "// @generated by `cargo xtask rewrite-derivability --emit-residue`. Do not edit.\n\
         //\n\
         // VERB_META: (lemma, principal-part metadata) for every verb lemma in\n\
         // data/extracted, encoded per the facade's `VerbMeta` field code tables\n\
         // (church-slavonic/src/lib.rs). VERB_RESIDUE: (lemma, verb cell code,\n\
         // variants) for exactly the attested cells the rule kernel (identity\n\
         // kernels + this metadata) does not reproduce verbatim. Both slices are\n\
         // sorted for binary search.\n",
    );
    let _ = writeln!(
        out,
        "pub static VERB_META: &[(&str, crate::VerbMeta<'static>)] = &[ // {} lemmas",
        oracle.meta.len()
    );
    for (lemma, codes) in &oracle.meta {
        let _ = write!(
            out,
            "    ({lemma:?}, crate::VerbMeta {{ aspect: {}, present: &[",
            codes.aspect
        );
        for (index, (stem, class, first, third, formation)) in codes.present.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(
                out,
                "({stem:?}, {class}, {first:?}, {third:?}, {formation})"
            );
        }
        out.push_str("], imperfect: &[");
        for (index, (stem, formation, policy)) in codes.imperfect.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "({stem:?}, {formation}, {policy})");
        }
        out.push_str("], aorist: &[");
        for (index, (stem, singular, formation)) in codes.aorist.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "({stem:?}, {singular:?}, {formation})");
        }
        out.push_str("], imperative: ");
        write_str_pair_slice(&mut out, &codes.imperative);
        out.push_str(", l_participle: &[");
        for (index, stem) in codes.l_participle.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{stem:?}");
        }
        out.push_str("], present_active_participle: ");
        write_str_pair_slice(&mut out, &codes.present_active_participle);
        out.push_str(", present_passive_participle: ");
        write_str_pair_slice(&mut out, &codes.present_passive_participle);
        out.push_str(", past_active_participle: ");
        write_str_pair_slice(&mut out, &codes.past_active_participle);
        out.push_str(", past_passive_participle: ");
        write_str_pair_slice(&mut out, &codes.past_passive_participle);
        out.push_str(" }),\n");
    }
    out.push_str("];\n");
    let _ = writeln!(
        out,
        "pub static VERB_RESIDUE: &[(&str, u8, &[&str])] = &[ // {} cells",
        residue.len()
    );
    for (lemma, code, variants) in &residue {
        let _ = write!(out, "    ({lemma:?}, {code}, &[");
        for (index, variant) in variants.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{variant:?}");
        }
        out.push_str("]),\n");
    }
    out.push_str("];\n");
    let path = root.join("crates/church-slavonic/generated/verb_residue.rs");
    fs::write(&path, &out)?;
    println!(
        "wrote {} ({} bytes): {} metadata rows, {} residue cells (of {} per-lexeme cells; \
         {} homograph groups)",
        path.display(),
        out.len(),
        oracle.meta.len(),
        residue.len(),
        oracle.cells.len(),
        oracle.homographs.len(),
    );
    Ok(())
}

/// Closed-class (pron/num/det) oracle at lemma granularity. Every lemma has
/// exactly one lexeme entry, so no homograph merge arises; cells still go
/// through the same rank-stable dedupe as the other POS.
struct ClosedOracle {
    /// lemma -> (pos code: 1 pron / 2 num / 3 det, shape flags: 1 bare /
    /// 2 gendered / 4 person-indexed cells attested).
    meta: BTreeMap<String, (u8, u8)>,
    /// (lemma, closed cell code) -> variant list in rank order.
    cells: BTreeMap<(String, u8), Vec<String>>,
    /// Extracted lexeme id -> lemma key (closed lemmas have one lexeme each).
    key_by_lexeme: BTreeMap<String, String>,
}

fn closed_pos_code(pos: &str) -> Option<u8> {
    match pos {
        "pron" => Some(1),
        "num" => Some(2),
        "det" => Some(3),
        _ => None,
    }
}

fn closed_pos_from_code(code: u8) -> PartOfSpeech {
    match code {
        1 => PartOfSpeech::Pronoun,
        2 => PartOfSpeech::Numeral,
        _ => PartOfSpeech::Determiner,
    }
}

/// Decode a closed cell code back into its typed dimensions (inverse of the
/// facade's `closed_cell_code`).
fn closed_cell_from_code(code: u8) -> (Case, Number, Option<Gender>, Option<Person>) {
    let (case, number) = cell_from_code(code / 6);
    let (gender, person) = match code % 6 {
        0 => (None, None),
        1 => (Some(Gender::Masculine), None),
        2 => (Some(Gender::Feminine), None),
        3 => (Some(Gender::Neuter), None),
        4 => (None, Some(Person::First)),
        _ => (None, Some(Person::Second)),
    };
    (case, number, gender, person)
}

fn load_closed_oracle(root: &Path) -> Result<ClosedOracle, Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    let mut meta: BTreeMap<String, (u8, u8)> = BTreeMap::new();
    let mut lemma_by_id: BTreeMap<&str, &str> = BTreeMap::new();
    for lexeme in &registry.lexemes {
        let Some(pos) = closed_pos_code(&lexeme.pos) else {
            continue;
        };
        lemma_by_id.insert(&lexeme.id, &lexeme.lemma);
        let previous = meta.insert(lexeme.lemma.clone(), (pos, 0));
        if let Some((previous_pos, _)) = previous {
            if previous_pos != pos {
                return Err(format!(
                    "closed-class lemma {} attested under two parts of speech",
                    lexeme.lemma
                )
                .into());
            }
        }
    }
    let mut rows: BTreeMap<(String, u8), Vec<(u16, String)>> = BTreeMap::new();
    for row in &registry.forms {
        let Some(lemma) = lemma_by_id.get(row.lexeme_id.as_str()) else {
            continue;
        };
        let parts: Vec<&str> = row.feature.split(':').collect();
        let ["decl", _, case, number, rest @ ..] = parts.as_slice() else {
            return Err(format!("unparsed closed-class feature {}", row.feature).into());
        };
        let (Some(case), Some(number)) = (crate::parse_case(case), crate::parse_number(number))
        else {
            return Err(format!("unparsed closed-class feature {}", row.feature).into());
        };
        let mut gender = None;
        let mut person = None;
        for value in rest.iter().copied() {
            if let Some(value) = crate::parse_gender_code(value) {
                gender = Some(value);
            } else if let Some(value) = crate::parse_person(value) {
                person = Some(value);
            } else {
                return Err(format!("unparsed closed-class feature {}", row.feature).into());
            }
        }
        let Some(code) = closed_cell_code(case, number, gender, person) else {
            return Err(format!("unencodable closed-class feature {}", row.feature).into());
        };
        let shape = match (gender, person) {
            (None, None) => 1u8,
            (Some(_), None) => 2,
            _ => 4,
        };
        meta.get_mut(*lemma).expect("lexeme meta").1 |= shape;
        rows.entry(((*lemma).to_string(), code))
            .or_default()
            .push((row.rank, row.form.clone()));
    }
    let mut cells: BTreeMap<(String, u8), Vec<String>> = BTreeMap::new();
    for ((lemma, code), mut ranked) in rows {
        ranked.sort_by_key(|(rank, _)| *rank);
        let mut texts: Vec<String> = Vec::new();
        for (_, form) in ranked {
            if !texts.contains(&form) {
                texts.push(form);
            }
        }
        cells.insert((lemma, code), texts);
    }
    let key_by_lexeme: BTreeMap<String, String> = lemma_by_id
        .iter()
        .map(|(id, lemma)| ((*id).to_string(), (*lemma).to_string()))
        .collect();
    Ok(ClosedOracle {
        meta,
        cells,
        key_by_lexeme,
    })
}

fn emit_closed_residue(root: &Path) -> Result<(), Box<dyn Error>> {
    let oracle = load_closed_oracle(root)?;
    let mut residue: Vec<(&str, u8, &Vec<String>)> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        let (case, number, gender, person) = closed_cell_from_code(*code);
        let pos = closed_pos_from_code(oracle.meta[lemma].0);
        let predicted = kernel_closed_variants(lemma, pos, case, number, gender, person);
        if predicted.as_deref() != Some(expected.as_slice()) {
            residue.push((lemma, *code, expected));
        }
    }
    let mut out = String::new();
    out.push_str(
        "// @generated by `cargo xtask rewrite-derivability --emit-residue`. Do not edit.\n\
         //\n\
         // CLOSED_META: (lemma, pos code 1 pron / 2 num / 3 det, shape flags 1 bare /\n\
         // 2 gendered / 4 person-indexed) for every closed-class lemma in\n\
         // data/extracted. CLOSED_RESIDUE: (lemma, closed cell code, variants) for\n\
         // exactly the attested cells the identity-kernel dispatch does not\n\
         // reproduce verbatim (the duplicated personal-pronoun tables under\n\
         // possessive and non-canonical personal lemmas, `етеръ`, `Єѵрѡпа`, and any\n\
         // kernel divergences). Both slices are sorted for binary search; see\n\
         // church-slavonic/src/lib.rs for the cell code layout.\n",
    );
    let _ = writeln!(
        out,
        "pub static CLOSED_META: &[(&str, u8, u8)] = &[ // {} lemmas",
        oracle.meta.len()
    );
    for (lemma, (pos, shape)) in &oracle.meta {
        let _ = writeln!(out, "    ({lemma:?}, {pos}, {shape}),");
    }
    out.push_str("];\n");
    let _ = writeln!(
        out,
        "pub static CLOSED_RESIDUE: &[(&str, u8, &[&str])] = &[ // {} cells",
        residue.len()
    );
    for (lemma, code, variants) in &residue {
        let _ = write!(out, "    ({lemma:?}, {code}, &[");
        for (index, variant) in variants.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{variant:?}");
        }
        out.push_str("]),\n");
    }
    out.push_str("];\n");
    let path = root.join("crates/church-slavonic/generated/closed_residue.rs");
    fs::write(&path, &out)?;
    println!(
        "wrote {} ({} bytes): {} metadata rows, {} residue cells (of {} cells)",
        path.display(),
        out.len(),
        oracle.meta.len(),
        residue.len(),
        oracle.cells.len(),
    );
    Ok(())
}

pub(crate) fn emit_residue(root: &Path) -> Result<(), Box<dyn Error>> {
    emit_closed_residue(root)?;
    emit_verb_residue(root)?;
    emit_adjective_residue(root)?;
    let oracle = load_noun_oracle(root)?;
    let mut residue: Vec<(&str, u8, &Vec<String>)> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        let meta = decode_meta(oracle.meta[lemma]);
        let (case, number) = cell_from_code(*code);
        let predicted =
            kernel_noun_variants(church_slavonic::base_lemma(lemma), &meta, case, number);
        if predicted.as_deref() != Some(expected.as_slice()) {
            residue.push((lemma, *code, expected));
        }
    }
    let mut out = String::new();
    out.push_str(
        "// @generated by `cargo xtask rewrite-derivability --emit-residue`. Do not edit.\n\
         //\n\
         // NOUN_META: (lemma, class, gender, animacy, number restriction) codes for\n\
         // every noun lemma in data/extracted (see church-slavonic/src/lib.rs for\n\
         // the code tables). NOUN_RESIDUE: (lemma, cell code, variants) for exactly\n\
         // the attested cells the rule kernel does not reproduce verbatim. Both\n\
         // slices are sorted for binary search.\n",
    );
    let _ = writeln!(
        out,
        "pub static NOUN_META: &[(&str, u8, u8, u8, u8)] = &[ // {} lemmas",
        oracle.meta.len()
    );
    for (lemma, (class, gender, animacy, restriction)) in &oracle.meta {
        let _ = writeln!(
            out,
            "    ({lemma:?}, {class}, {gender}, {animacy}, {restriction}),"
        );
    }
    out.push_str("];\n");
    let _ = writeln!(
        out,
        "pub static NOUN_RESIDUE: &[(&str, u8, &[&str])] = &[ // {} cells",
        residue.len()
    );
    for (lemma, code, variants) in &residue {
        let _ = write!(out, "    ({lemma:?}, {code}, &[");
        for (index, variant) in variants.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{variant:?}");
        }
        out.push_str("]),\n");
    }
    out.push_str("];\n");
    let path = root.join("crates/church-slavonic/generated/noun_residue.rs");
    fs::write(&path, &out)?;
    println!(
        "wrote {} ({} bytes): {} metadata rows, {} residue cells (of {} per-lexeme cells; \
         {} homograph groups)",
        path.display(),
        out.len(),
        oracle.meta.len(),
        residue.len(),
        oracle.cells.len(),
        oracle.homographs.len(),
    );
    Ok(())
}

fn print_homographs(homographs: &[(String, Vec<String>)]) {
    println!(
        "  homograph groups: {} (deterministic numeric-suffix keys; bare lemma = default sense)",
        homographs.len()
    );
    for (lemma, keys) in homographs {
        println!("    {lemma}: {}", keys.join(", "));
    }
}

pub(crate) fn accuracy(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(extra) = args.next() {
        return Err(format!("rewrite-pilot-accuracy takes no arguments, found {extra}").into());
    }
    let oracle = load_noun_oracle(root)?;
    let mut total = 0usize;
    let mut matched = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        total += 1;
        let (case, number) = cell_from_code(*code);
        match church_slavonic::noun_variants(lemma, case, number) {
            Ok(variants) if &variants == expected => matched += 1,
            other => {
                if mismatches.len() < 20 {
                    mismatches.push(format!(
                        "{lemma} cell {code}: stored {:?} vs facade {other:?}",
                        expected
                    ));
                }
            }
        }
    }
    let generated = root.join("crates/church-slavonic/generated/noun_residue.rs");
    let bytes = fs::metadata(&generated)?.len();
    println!("rewrite pilot accuracy (nouns, per-lexeme oracle, suffixed homograph keys)");
    println!("  per-lexeme cells matched: {matched}/{total}");
    print_homographs(&oracle.homographs);
    println!(
        "  generated table size: {bytes} bytes ({generated})",
        generated = generated.display()
    );
    for line in &mismatches {
        println!("  MISMATCH {line}");
    }
    if matched != total {
        return Err(format!("pilot accuracy {matched}/{total}, expected 100%").into());
    }

    let adjectives = load_adjective_oracle(root)?;
    let mut adj_total = 0usize;
    let mut adj_matched = 0usize;
    let mut adj_mismatches: Vec<String> = Vec::new();
    for ((lemma, code), expected) in &adjectives.cells {
        adj_total += 1;
        let (form, case, number, gender) = adjective_cell_from_code(*code);
        let produced = match form {
            AdjectiveForm::Long => church_slavonic::adjective_variants(lemma, case, number, gender),
            AdjectiveForm::Short => {
                church_slavonic::short_adjective_variants(lemma, case, number, gender)
            }
        };
        match produced {
            Ok(variants) if &variants == expected => adj_matched += 1,
            other => {
                if adj_mismatches.len() < 20 {
                    adj_mismatches.push(format!(
                        "{lemma} cell {code}: stored {expected:?} vs facade {other:?}"
                    ));
                }
            }
        }
    }
    let adj_generated = root.join("crates/church-slavonic/generated/adjective_residue.rs");
    let adj_bytes = fs::metadata(&adj_generated)?.len();
    println!(
        "rewrite pilot accuracy (adjectives, per-lexeme oracle, animacy collapsed, \
         suffixed homograph keys)"
    );
    println!("  per-lexeme cells matched: {adj_matched}/{adj_total}");
    println!(
        "  keyed (lexeme, feature) cells: {} of which {} comparative-citation cells are \
         excluded from the facade (unpredictable suffix grade / suppletion)",
        adjectives.keyed_cells, adjectives.comparative_cells
    );
    print_homographs(&adjectives.homographs);
    println!(
        "  generated table size: {adj_bytes} bytes ({path})",
        path = adj_generated.display()
    );
    for line in &adj_mismatches {
        println!("  MISMATCH {line}");
    }
    if adj_matched != adj_total {
        return Err(
            format!("adjective pilot accuracy {adj_matched}/{adj_total}, expected 100%").into(),
        );
    }

    let verbs = load_verb_oracle(root)?;
    let mut verb_total = 0usize;
    let mut verb_matched = 0usize;
    let mut verb_rules_cells = 0usize;
    let mut verb_mismatches: Vec<String> = Vec::new();
    for ((lemma, code), expected) in &verbs.cells {
        verb_total += 1;
        let cell = verb_cell_from_code(*code);
        let produced = match cell {
            VerbCell::Finite(finite) => match finite.tense {
                FiniteTense::Present => {
                    church_slavonic::present_variants(lemma, finite.person, finite.number)
                }
                FiniteTense::Imperfect => {
                    church_slavonic::imperfect_variants(lemma, finite.person, finite.number)
                }
                FiniteTense::Aorist => {
                    church_slavonic::aorist_variants(lemma, finite.person, finite.number)
                }
            },
            VerbCell::Imperative(imperative) => {
                church_slavonic::imperative_variants(lemma, imperative.person, imperative.number)
            }
            VerbCell::LParticiple(participle) => {
                church_slavonic::l_participle_variants(lemma, participle.gender, participle.number)
            }
            VerbCell::Infinitive => church_slavonic::infinitive_variants(lemma),
            VerbCell::Supine => church_slavonic::supine_variants(lemma),
            VerbCell::VerbalNoun => church_slavonic::verbal_noun_variants(lemma),
            VerbCell::ParticipleCitation(kind) => match kind {
                ParticipleKind::PresentActive => {
                    church_slavonic::present_active_participle_variants(lemma)
                }
                ParticipleKind::PresentPassive => {
                    church_slavonic::present_passive_participle_variants(lemma)
                }
                ParticipleKind::PastActive => {
                    church_slavonic::past_active_participle_variants(lemma)
                }
                ParticipleKind::PastPassive => {
                    church_slavonic::past_passive_participle_variants(lemma)
                }
            },
        };
        // The rules-vs-residue split: a cell is rules-served exactly when the
        // shared kernel reproduces the stored list (the emitter's criterion),
        // measured over the shipped `VERB_META` table so synthesized
        // principal-part metadata counts as rules.
        let kernel = church_slavonic::verb_meta(lemma)
            .and_then(|meta| kernel_verb_variants(church_slavonic::base_lemma(lemma), &meta, cell));
        if kernel.as_deref() == Some(expected.as_slice()) {
            verb_rules_cells += 1;
        }
        match produced {
            Ok(variants) if &variants == expected => verb_matched += 1,
            other => {
                if verb_mismatches.len() < 20 {
                    verb_mismatches.push(format!(
                        "{lemma} cell {code}: stored {expected:?} vs facade {other:?}"
                    ));
                }
            }
        }
    }
    let verb_generated = root.join("crates/church-slavonic/generated/verb_residue.rs");
    let verb_bytes = fs::metadata(&verb_generated)?.len();
    println!("rewrite pilot accuracy (verbs, per-lexeme oracle, suffixed homograph keys)");
    println!("  per-lexeme cells matched: {verb_matched}/{verb_total}");
    println!(
        "  rules-vs-residue split: {verb_rules_cells} cells from the rule kernel          (identity kernels + principal-part metadata), {} from the residue table",
        verb_total - verb_rules_cells
    );
    print_homographs(&verbs.homographs);
    println!(
        "  generated table size: {verb_bytes} bytes ({path})",
        path = verb_generated.display()
    );
    for line in &verb_mismatches {
        println!("  MISMATCH {line}");
    }
    if verb_matched != verb_total {
        return Err(
            format!("verb pilot accuracy {verb_matched}/{verb_total}, expected 100%").into(),
        );
    }

    let closed = load_closed_oracle(root)?;
    // Per POS: (matched, total, kernel-served cells).
    let mut closed_counts: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    let mut closed_mismatches: Vec<String> = Vec::new();
    for ((lemma, code), expected) in &closed.cells {
        let (pos_code, _shape) = closed.meta[lemma];
        let pos_name = match pos_code {
            1 => "pron",
            2 => "num",
            _ => "det",
        };
        let counts = closed_counts.entry(pos_name).or_default();
        counts.1 += 1;
        let (case, number, gender, person) = closed_cell_from_code(*code);
        let pos = closed_pos_from_code(pos_code);
        // The rules-vs-residue split: a cell is kernel-served exactly when
        // the shared identity-kernel dispatch reproduces the stored list
        // (the emitter's criterion).
        if kernel_closed_variants(lemma, pos, case, number, gender, person).as_deref()
            == Some(expected.as_slice())
        {
            counts.2 += 1;
        }
        // Lemma-keyed resolution (residue table -> identity kernels), the
        // path every public wrapper goes through.
        let lemma_route =
            church_slavonic::closed_variants(lemma, pos, case, number, gender, person);
        // Public API route for this cell's shape.
        let public_route = match (pos_code, gender, person) {
            (1, None, Some(person)) => church_slavonic::pronoun_variants(person, number, case),
            (1, None, None) if lemma == "сѧ" => church_slavonic::reflexive_variants(case),
            (1, Some(gender), None) => {
                church_slavonic::pronoun_form_variants(lemma, case, number, gender)
            }
            (1, None, None) => {
                church_slavonic::pronoun_form_variants(lemma, case, number, Gender::Masculine)
            }
            (2, gender, None) => church_slavonic::numeral_form_variants(
                lemma,
                case,
                number,
                gender.unwrap_or(Gender::Masculine),
            ),
            (3, gender, None) => church_slavonic::determiner_form_variants(
                lemma,
                case,
                number,
                gender.unwrap_or(Gender::Masculine),
            ),
            _ => Err(church_slavonic::Error::Underdetermined {
                lemma: lemma.to_string(),
            }),
        };
        let lemma_ok = lemma_route.as_ref().map(Vec::as_slice) == Ok(expected.as_slice());
        let public_ok = public_route.as_ref().map(Vec::as_slice) == Ok(expected.as_slice());
        if lemma_ok && public_ok {
            counts.0 += 1;
        } else if closed_mismatches.len() < 20 {
            closed_mismatches.push(format!(
                "{lemma} cell {code}: stored {expected:?} vs lemma-keyed {lemma_route:?} / \
                 public {public_route:?}"
            ));
        }
    }
    let closed_generated = root.join("crates/church-slavonic/generated/closed_residue.rs");
    let closed_bytes = fs::metadata(&closed_generated)?.len();
    println!(
        "rewrite pilot accuracy (closed classes, lemma-keyed oracle; both the lemma-keyed \
         resolution and the public API route must match)"
    );
    for (pos_name, (matched, total, kernel_cells)) in &closed_counts {
        println!(
            "  {pos_name}: merged cells matched: {matched}/{total} (rules-vs-residue split: \
             {kernel_cells} cells from the identity kernels, {} from the residue table)",
            total - kernel_cells
        );
    }
    println!(
        "  generated table size: {closed_bytes} bytes ({path})",
        path = closed_generated.display()
    );
    for line in &closed_mismatches {
        println!("  MISMATCH {line}");
    }
    for (pos_name, (matched, total, _)) in &closed_counts {
        if matched != total {
            return Err(
                format!("{pos_name} pilot accuracy {matched}/{total}, expected 100%").into(),
            );
        }
    }
    paradigm_consistency(&oracle, &adjectives, &verbs, &closed)?;
    numeral_value_differential()?;
    participle_differential()?;
    phrase_differential()?;
    Ok(())
}

/// Differential gate for the pilot's declined participles: the fat
/// old-church-slavonic facade's participle handles are the reference (no
/// attested oracle exists — extraction excluded declined participles as not
/// safely attributed). Over a deterministic sweep of a representative verb
/// set (regular metadata-served, §104 irregular, and reviewed unique
/// identities) x all four kinds x case x number x gender x long/short form
/// (inanimate convention, matching the pilot adjective surface), the pilot's
/// `participle()` primary must equal the old handle's declined cell primary,
/// with agreement on rejected cells counted, at 100%.
fn participle_differential() -> Result<(), Box<dyn Error>> {
    // Regular metadata-served verbs, §104 irregulars, reviewed unique
    // identities.
    const VERBS: [&str; 14] = [
        "благословити",
        "любити",
        "творити",
        "глаголати",
        "нести",
        "рещи",
        "видѣти",
        "бити",
        "метати",
        "пѣти",
        "бꙑти",
        "ити",
        "имѣти",
        "хотѣти",
    ];
    let old_handle = |lemma: &str, kind: ParticipleKind| match kind {
        ParticipleKind::PresentActive => old_church_slavonic::present_active_participle(lemma),
        ParticipleKind::PresentPassive => old_church_slavonic::present_passive_participle(lemma),
        ParticipleKind::PastActive => old_church_slavonic::past_active_participle(lemma),
        ParticipleKind::PastPassive => old_church_slavonic::past_passive_participle(lemma),
    };
    let mut agreements = 0usize;
    let mut total = 0usize;
    let mut rejected_pairs = 0usize;
    let mut citation_cells = 0usize;
    let mut citation_old_agreements = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    for lemma in VERBS {
        for kind in [
            ParticipleKind::PresentActive,
            ParticipleKind::PresentPassive,
            ParticipleKind::PastActive,
            ParticipleKind::PastPassive,
        ] {
            let handle = old_handle(lemma, kind).ok();
            for case in Case::ALL {
                for number in Number::ALL {
                    for gender in Gender::ALL {
                        for form in [AdjectiveForm::Long, AdjectiveForm::Short] {
                            total += 1;
                            let old = handle.as_ref().and_then(|participle| {
                                match form {
                                    AdjectiveForm::Long => {
                                        participle.long(case, number, gender, Animacy::Inanimate)
                                    }
                                    AdjectiveForm::Short => {
                                        participle.short(case, number, gender, Animacy::Inanimate)
                                    }
                                }
                                .ok()
                                .map(|forms| forms.primary_text().to_string())
                            });
                            let new = church_slavonic::participle(
                                lemma, kind, case, number, gender, form,
                            )
                            .ok();
                            // The citation cell (short nominative singular
                            // masculine) is governed by the pilot's
                            // attested-first citation precedence (main
                            // accuracy gate), not the old facade's
                            // reviewed-first dispatch, so it is asserted
                            // against the pilot's own citation function
                            // instead; old-facade agreement there is
                            // reported but not gated.
                            let is_citation = case == Case::Nominative
                                && number == Number::Singular
                                && gender == Gender::Masculine
                                && form == AdjectiveForm::Short;
                            if is_citation {
                                citation_cells += 1;
                                if new == old {
                                    citation_old_agreements += 1;
                                }
                                let citation = match kind {
                                    ParticipleKind::PresentActive => {
                                        church_slavonic::present_active_participle(lemma)
                                    }
                                    ParticipleKind::PresentPassive => {
                                        church_slavonic::present_passive_participle(lemma)
                                    }
                                    ParticipleKind::PastActive => {
                                        church_slavonic::past_active_participle(lemma)
                                    }
                                    ParticipleKind::PastPassive => {
                                        church_slavonic::past_passive_participle(lemma)
                                    }
                                }
                                .ok();
                                let consistent = match &citation {
                                    Some(text) => new.as_deref() == Some(text.as_str()),
                                    None => true,
                                };
                                if consistent {
                                    agreements += 1;
                                } else if mismatches.len() < 30 {
                                    mismatches.push(format!(
                                        "{lemma} {kind:?} citation: participle() {new:?} vs \
                                         citation function {citation:?}"
                                    ));
                                }
                                continue;
                            }
                            match (&new, &old) {
                                (Some(new_text), Some(old_text)) if new_text == old_text => {
                                    agreements += 1;
                                }
                                (None, None) => {
                                    agreements += 1;
                                    rejected_pairs += 1;
                                }
                                _ => {
                                    if mismatches.len() < 30 {
                                        mismatches.push(format!(
                                            "{lemma} {kind:?} {case:?} {number:?} {gender:?} \
                                             {form:?}: new {new:?} vs old {old:?}"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "rewrite pilot declined-participle differential (pilot participle() vs the old facade \
         participle handles; both-rejected counts as agreement)"
    );
    println!(
        "  {agreements}/{total} sweep cells agree ({rejected_pairs} agreed-rejected, \
         {} verbs x 4 kinds x case x number x gender x form)",
        VERBS.len()
    );
    println!(
        "  citation cells ({citation_cells}) are gated as self-consistency with the pilot's \
         attested-first citation functions; {citation_old_agreements}/{citation_cells} also \
         agree with the old facade's reviewed-first dispatch"
    );
    for line in &mismatches {
        println!("  MISMATCH {line}");
    }
    if agreements != total {
        return Err("declined-participle differential disagreements, expected 100%".into());
    }
    Ok(())
}

/// Differential gate for the pilot's phrase layer: the fat old-church-
/// slavonic facade's `phrases` module is the reference. Over a deterministic
/// parameter sweep every kept construction must agree on the primary phrase
/// text — or agree that the request is rejected — at 100%. The absolute
/// superlative is gated against the old facade's lemma-keyed
/// `long_adjective`/`short_adjective` joined with the invariant adverb per
/// the same order convention (the old phrase function takes a bare
/// `AdjectiveLexeme`, bypassing the lemma-keyed dictionary route that is the
/// actual behavioural contract).
fn phrase_differential() -> Result<(), Box<dyn Error>> {
    use church_slavonic::phrases as new_phrases;
    use old_church_slavonic::phrases as old_phrases;
    use old_church_slavonic_core::{
        ConditionalAuxiliary, CopulaSeries, DirectToTreatment, FutureInfinitiveAuxiliary,
        FutureReferenceTense, ImpersonalVerbIdentity, InterrogativePronounIdentity,
        PassiveAuxiliary, PhraseOrder, PluperfectAuxiliary, PronominalFamilySpec,
        PronominalPostpositive, PronominalPrefix,
    };

    const VERBS: [&str; 3] = ["благословити", "любити", "творити"];
    const ADJECTIVES: [&str; 3] = ["новъ", "свѧтъ", "добръ"];
    const ORDERS: [PhraseOrder; 2] = [PhraseOrder::DependentFirst, PhraseOrder::HeadFirst];

    let mut counts: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    let mut mismatches: Vec<String> = Vec::new();
    let mut check =
        |construction: &'static str, detail: String, new: Option<String>, old: Option<String>| {
            let entry = counts.entry(construction).or_insert((0, 0));
            entry.1 += 1;
            match (&new, &old) {
                (Some(new_text), Some(old_text)) if new_text == old_text => entry.0 += 1,
                (None, None) => entry.0 += 1,
                _ => {
                    if mismatches.len() < 30 {
                        mismatches.push(format!(
                            "{construction} {detail}: new {new:?} vs old {old:?}"
                        ));
                    }
                }
            }
        };

    // §316 derived pronominal families: every prefix x postpositive x
    // direct-то treatment x interposed-preposition choice over both
    // interrogative bases and all seven cases.
    for (identity, lemma) in [
        (InterrogativePronounIdentity::Kto, "къто"),
        (InterrogativePronounIdentity::Chto, "чьто"),
    ] {
        for case in Case::ALL {
            for prefix in [None, Some(PronominalPrefix::Ni), Some(PronominalPrefix::Ne)] {
                for postpositive in [
                    None,
                    Some(PronominalPostpositive::Ze),
                    Some(PronominalPostpositive::Zhde),
                    Some(PronominalPostpositive::Zhydo),
                    Some(PronominalPostpositive::Liubo),
                ] {
                    for direct_to in [
                        None,
                        Some(DirectToTreatment::Retain),
                        Some(DirectToTreatment::Drop),
                    ] {
                        for preposition in [None, Some("о")] {
                            let old = old_phrases::interrogative_pronoun_family(
                                identity,
                                case,
                                PronominalFamilySpec {
                                    prefix,
                                    postpositive,
                                    direct_to,
                                    preposition: preposition.map(str::to_string),
                                },
                            )
                            .ok()
                            .map(|phrase| phrase.primary_text());
                            let new = new_phrases::pronominal_family(
                                lemma,
                                case,
                                prefix,
                                postpositive,
                                direct_to,
                                preposition,
                            )
                            .ok();
                            check(
                                "pronominal_family",
                                format!(
                                    "{lemma} {case:?} {prefix:?} {postpositive:?} {direct_to:?} {preposition:?}"
                                ),
                                new,
                                old,
                            );
                        }
                    }
                }
            }
        }
    }

    // Absolute superlative, both adjective forms and both orders.
    for lemma in ADJECTIVES {
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    for order in ORDERS {
                        let join = |adjective: String| match order {
                            PhraseOrder::DependentFirst => format!("ѕѣло {adjective}"),
                            PhraseOrder::HeadFirst => format!("{adjective} ѕѣло"),
                        };
                        let old_long = old_church_slavonic::long_adjective(
                            lemma,
                            case,
                            number,
                            gender,
                            Animacy::Inanimate,
                        )
                        .ok()
                        .map(|forms| join(forms.primary_text().to_string()));
                        let new_long =
                            new_phrases::absolute_superlative(lemma, case, number, gender, order)
                                .ok();
                        check(
                            "absolute_superlative",
                            format!("{lemma} {case:?} {number:?} {gender:?} {order:?}"),
                            new_long,
                            old_long,
                        );
                        let old_short = old_church_slavonic::short_adjective(
                            lemma,
                            case,
                            number,
                            gender,
                            Animacy::Inanimate,
                        )
                        .ok()
                        .map(|forms| join(forms.primary_text().to_string()));
                        let new_short = new_phrases::short_absolute_superlative(
                            lemma, case, number, gender, order,
                        )
                        .ok();
                        check(
                            "short_absolute_superlative",
                            format!("{lemma} {case:?} {number:?} {gender:?} {order:?}"),
                            new_short,
                            old_short,
                        );
                    }
                }
            }
        }
    }

    // Copular series: all six series over every person-number cell.
    for series in CopulaSeries::ALL {
        for number in Number::ALL {
            for person in Person::ALL {
                let old = old_phrases::copula(series, person, number)
                    .ok()
                    .map(|forms| forms.primary_text().to_string());
                let new = Some(match series {
                    CopulaSeries::PresentEs => new_phrases::copula_present(person, number),
                    CopulaSeries::FutureBud => new_phrases::copula_future(person, number),
                    CopulaSeries::ImperfectBe => new_phrases::copula_imperfect(person, number),
                    CopulaSeries::AoristBe => new_phrases::copula_aorist(person, number),
                    CopulaSeries::ConditionalBi => new_phrases::copula_conditional(person, number),
                    CopulaSeries::ConditionalAoristBy => {
                        new_phrases::copula_conditional_aorist(person, number)
                    }
                });
                check(
                    "copula",
                    format!("{series:?} {person:?} {number:?}"),
                    new,
                    old,
                );
            }
        }
    }

    for lemma in VERBS {
        for number in Number::ALL {
            for person in Person::ALL {
                // да + present.
                let old = old_phrases::da_imperative(lemma, person, number)
                    .ok()
                    .map(|phrase| phrase.primary_text());
                let new = new_phrases::da_imperative(lemma, person, number).ok();
                check(
                    "da_imperative",
                    format!("{lemma} {person:?} {number:?}"),
                    new,
                    old,
                );

                // Infinitival futures over every auxiliary and reference tense.
                for auxiliary in FutureInfinitiveAuxiliary::ALL {
                    for tense in [
                        FutureReferenceTense::Present,
                        FutureReferenceTense::Imperfect,
                        FutureReferenceTense::Aorist,
                    ] {
                        for order in ORDERS {
                            let old = old_phrases::infinitival_future(
                                lemma, auxiliary, tense, person, number, order,
                            )
                            .ok()
                            .map(|phrase| phrase.primary_text());
                            let new = match tense {
                                FutureReferenceTense::Present => new_phrases::infinitival_future(
                                    lemma, auxiliary, person, number, order,
                                ),
                                FutureReferenceTense::Imperfect => {
                                    new_phrases::infinitival_future_imperfect(
                                        lemma, auxiliary, person, number, order,
                                    )
                                }
                                FutureReferenceTense::Aorist => {
                                    new_phrases::infinitival_future_aorist(
                                        lemma, auxiliary, person, number, order,
                                    )
                                }
                            }
                            .ok();
                            check(
                                "infinitival_future",
                                format!(
                                    "{lemma} {auxiliary:?} {tense:?} {person:?} {number:?} {order:?}"
                                ),
                                new,
                                old,
                            );
                        }
                    }
                }

                for gender in Gender::ALL {
                    for order in ORDERS {
                        // Perfect and future perfect.
                        let old = old_phrases::perfect(lemma, person, number, gender, order)
                            .ok()
                            .map(|phrase| phrase.primary_text());
                        let new = new_phrases::perfect(lemma, person, number, gender, order).ok();
                        check(
                            "perfect",
                            format!("{lemma} {person:?} {number:?} {gender:?} {order:?}"),
                            new,
                            old,
                        );
                        let old = old_phrases::future_perfect(lemma, person, number, gender, order)
                            .ok()
                            .map(|phrase| phrase.primary_text());
                        let new =
                            new_phrases::future_perfect(lemma, person, number, gender, order).ok();
                        check(
                            "future_perfect",
                            format!("{lemma} {person:?} {number:?} {gender:?} {order:?}"),
                            new,
                            old,
                        );

                        // The three pluperfect formations.
                        for auxiliary in [
                            PluperfectAuxiliary::Imperfect,
                            PluperfectAuxiliary::Aorist,
                            PluperfectAuxiliary::Perfect,
                        ] {
                            let old = old_phrases::pluperfect(
                                lemma, person, number, gender, auxiliary, order,
                            )
                            .ok()
                            .map(|phrase| phrase.primary_text());
                            let new = match auxiliary {
                                PluperfectAuxiliary::Imperfect => {
                                    new_phrases::pluperfect(lemma, person, number, gender, order)
                                }
                                PluperfectAuxiliary::Aorist => new_phrases::pluperfect_aorist(
                                    lemma, person, number, gender, order,
                                ),
                                PluperfectAuxiliary::Perfect => new_phrases::pluperfect_perfect(
                                    lemma, person, number, gender, order,
                                ),
                            }
                            .ok();
                            check(
                                "pluperfect",
                                format!(
                                    "{lemma} {person:?} {number:?} {gender:?} {auxiliary:?} {order:?}"
                                ),
                                new,
                                old,
                            );
                        }

                        // Conditional-optatives, plain and да-marked.
                        for auxiliary in [
                            ConditionalAuxiliary::Conditional,
                            ConditionalAuxiliary::AoristReplacement,
                        ] {
                            let old = old_phrases::conditional_optative(
                                lemma, person, number, gender, auxiliary, order,
                            )
                            .ok()
                            .map(|phrase| phrase.primary_text());
                            let new = match auxiliary {
                                ConditionalAuxiliary::Conditional => {
                                    new_phrases::conditional_optative(
                                        lemma, person, number, gender, order,
                                    )
                                }
                                ConditionalAuxiliary::AoristReplacement => {
                                    new_phrases::conditional_optative_aorist(
                                        lemma, person, number, gender, order,
                                    )
                                }
                            }
                            .ok();
                            check(
                                "conditional_optative",
                                format!(
                                    "{lemma} {person:?} {number:?} {gender:?} {auxiliary:?} {order:?}"
                                ),
                                new,
                                old,
                            );
                            let old = old_phrases::da_conditional_optative(
                                lemma, person, number, gender, auxiliary, order,
                            )
                            .ok()
                            .map(|phrase| phrase.primary_text());
                            let new = match auxiliary {
                                ConditionalAuxiliary::Conditional => {
                                    new_phrases::da_conditional_optative(
                                        lemma, person, number, gender, order,
                                    )
                                }
                                ConditionalAuxiliary::AoristReplacement => {
                                    new_phrases::da_conditional_optative_aorist(
                                        lemma, person, number, gender, order,
                                    )
                                }
                            }
                            .ok();
                            check(
                                "da_conditional_optative",
                                format!(
                                    "{lemma} {person:?} {number:?} {gender:?} {auxiliary:?} {order:?}"
                                ),
                                new,
                                old,
                            );
                        }
                    }
                }
            }
        }
    }

    // Declined-participle predicates: the analytic passive (each copular
    // auxiliary a pilot function), the conditional passive, and the
    // participial future, over every participle kind (exercising the
    // active/passive licensing rejections), person x number x gender x
    // order. The old functions take a caller-supplied AdjectiveCell; the
    // pilot derives the short nominative subject-agreeing cell, so the
    // sweep supplies exactly that cell to the old side.
    for lemma in VERBS {
        for kind in [
            ParticipleKind::PresentActive,
            ParticipleKind::PresentPassive,
            ParticipleKind::PastActive,
            ParticipleKind::PastPassive,
        ] {
            for person in [Person::First, Person::Second, Person::Third] {
                for number in [Number::Singular, Number::Dual, Number::Plural] {
                    for gender in [Gender::Masculine, Gender::Feminine, Gender::Neuter] {
                        let cell = old_church_slavonic_core::AdjectiveCell {
                            case: Case::Nominative,
                            number,
                            gender,
                            animacy: Animacy::Inanimate,
                            form: AdjectiveForm::Short,
                        };
                        for order in ORDERS {
                            for auxiliary in [
                                PassiveAuxiliary::Present,
                                PassiveAuxiliary::Imperfect,
                                PassiveAuxiliary::Aorist,
                                PassiveAuxiliary::Future,
                                PassiveAuxiliary::Conditional,
                                PassiveAuxiliary::ConditionalAoristReplacement,
                            ] {
                                let old = old_phrases::analytic_passive(
                                    lemma, kind, cell, person, number, auxiliary, order,
                                )
                                .ok()
                                .map(|phrase| phrase.primary_text());
                                let new = match auxiliary {
                                    PassiveAuxiliary::Present => new_phrases::analytic_passive(
                                        lemma, kind, person, number, gender, order,
                                    ),
                                    PassiveAuxiliary::Imperfect => {
                                        new_phrases::analytic_passive_imperfect(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                    PassiveAuxiliary::Aorist => {
                                        new_phrases::analytic_passive_aorist(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                    PassiveAuxiliary::Future => {
                                        new_phrases::analytic_passive_future(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                    PassiveAuxiliary::Conditional => {
                                        new_phrases::conditional_passive(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                    PassiveAuxiliary::ConditionalAoristReplacement => {
                                        new_phrases::conditional_passive_aorist(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                }
                                .ok();
                                check(
                                    "analytic_passive",
                                    format!(
                                        "{lemma} {kind:?} {person:?} {number:?} {gender:?} \
                                         {auxiliary:?} {order:?}"
                                    ),
                                    new,
                                    old,
                                );
                            }
                            for auxiliary in [
                                ConditionalAuxiliary::Conditional,
                                ConditionalAuxiliary::AoristReplacement,
                            ] {
                                let old = old_phrases::conditional_passive(
                                    lemma, kind, cell, person, number, auxiliary, order,
                                )
                                .ok()
                                .map(|phrase| phrase.primary_text());
                                let new = match auxiliary {
                                    ConditionalAuxiliary::Conditional => {
                                        new_phrases::conditional_passive(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                    ConditionalAuxiliary::AoristReplacement => {
                                        new_phrases::conditional_passive_aorist(
                                            lemma, kind, person, number, gender, order,
                                        )
                                    }
                                }
                                .ok();
                                check(
                                    "conditional_passive",
                                    format!(
                                        "{lemma} {kind:?} {person:?} {number:?} {gender:?} \
                                         {auxiliary:?} {order:?}"
                                    ),
                                    new,
                                    old,
                                );
                            }
                            let old = old_phrases::participial_future(
                                lemma, kind, cell, person, number, order,
                            )
                            .ok()
                            .map(|phrase| phrase.primary_text());
                            let new = new_phrases::participial_future(
                                lemma, kind, person, number, gender, order,
                            )
                            .ok();
                            check(
                                "participial_future",
                                format!(
                                    "{lemma} {kind:?} {person:?} {number:?} {gender:?} {order:?}"
                                ),
                                new,
                                old,
                            );
                        }
                    }
                }
            }
        }
    }

    // Impersonal predicates over every finite tense.
    for identity in ImpersonalVerbIdentity::ALL {
        for tense in FiniteTense::ALL {
            let old = old_phrases::impersonal_predicate(identity, tense)
                .ok()
                .map(|phrase| phrase.primary_text());
            let new = match tense {
                FiniteTense::Present => new_phrases::impersonal_present(identity.lemma()),
                FiniteTense::Imperfect => new_phrases::impersonal_imperfect(identity.lemma()),
                FiniteTense::Aorist => new_phrases::impersonal_aorist(identity.lemma()),
            }
            .ok();
            check(
                "impersonal_predicate",
                format!("{identity:?} {tense:?}"),
                new,
                old,
            );
        }
    }

    println!(
        "rewrite pilot phrase differential (pilot phrases module vs the old facade phrase \
         layer; both-rejected counts as agreement)"
    );
    let mut failed = false;
    for (construction, (agreements, total)) in &counts {
        println!("  {construction}: {agreements}/{total} sweep cells agree");
        if agreements != total {
            failed = true;
        }
    }
    for line in &mismatches {
        println!("  MISMATCH {line}");
    }
    if failed {
        return Err("phrase differential disagreements, expected 100%".into());
    }
    Ok(())
}

/// Paradigm self-consistency gate: for every attested lexeme, the enumerated
/// `*_paradigm` listing must contain exactly the cells the single-cell API
/// serves (no more, no fewer), in the enumeration order, and agree on every
/// variant list. Both sides are probed through the public API, so this is a
/// pure re-derivation with no second oracle.
fn paradigm_consistency(
    nouns: &NounOracle,
    adjectives: &AdjectiveOracle,
    verbs: &VerbOracle,
    closed: &ClosedOracle,
) -> Result<(), Box<dyn Error>> {
    use church_slavonic::VerbCellKind;

    fn grid() -> impl Iterator<Item = (Case, Number)> {
        Case::ALL
            .into_iter()
            .flat_map(|case| Number::ALL.into_iter().map(move |number| (case, number)))
    }

    let mut mismatches: Vec<String> = Vec::new();

    let noun_lemmas: BTreeSet<&String> = nouns.cells.keys().map(|(lemma, _)| lemma).collect();
    let mut noun_cells = 0usize;
    for lemma in &noun_lemmas {
        let expected: Vec<(Case, Number, Vec<String>)> = grid()
            .filter_map(|(case, number)| {
                church_slavonic::noun_variants(lemma, case, number)
                    .ok()
                    .map(|variants| (case, number, variants))
            })
            .collect();
        noun_cells += expected.len();
        if church_slavonic::noun_paradigm(lemma).as_ref() != Ok(&expected) && mismatches.len() < 20
        {
            mismatches.push(format!(
                "noun {lemma}: paradigm disagrees with single-cell API"
            ));
        }
    }

    let adjective_lemmas: BTreeSet<&String> =
        adjectives.cells.keys().map(|(lemma, _)| lemma).collect();
    let mut adjective_cells = 0usize;
    for lemma in &adjective_lemmas {
        for form in [AdjectiveForm::Long, AdjectiveForm::Short] {
            let expected: Vec<(Case, Number, Gender, Vec<String>)> = grid()
                .flat_map(|(case, number)| {
                    Gender::ALL
                        .into_iter()
                        .map(move |gender| (case, number, gender))
                })
                .filter_map(|(case, number, gender)| {
                    let produced = match form {
                        AdjectiveForm::Long => {
                            church_slavonic::adjective_variants(lemma, case, number, gender)
                        }
                        AdjectiveForm::Short => {
                            church_slavonic::short_adjective_variants(lemma, case, number, gender)
                        }
                    };
                    produced
                        .ok()
                        .map(|variants| (case, number, gender, variants))
                })
                .collect();
            adjective_cells += expected.len();
            if church_slavonic::adjective_paradigm(lemma, form).as_ref() != Ok(&expected)
                && mismatches.len() < 20
            {
                mismatches.push(format!(
                    "adjective {lemma} ({form:?}): paradigm disagrees with single-cell API"
                ));
            }
        }
    }

    let verb_lemmas: BTreeSet<&String> = verbs.cells.keys().map(|(lemma, _)| lemma).collect();
    let mut verb_cells = 0usize;
    for lemma in &verb_lemmas {
        let expected: Vec<(VerbCellKind, Vec<String>)> = VerbCellKind::all()
            .into_iter()
            .filter_map(|kind| {
                let produced = match kind {
                    VerbCellKind::Present { person, number } => {
                        church_slavonic::present_variants(lemma, person, number)
                    }
                    VerbCellKind::Imperfect { person, number } => {
                        church_slavonic::imperfect_variants(lemma, person, number)
                    }
                    VerbCellKind::Aorist { person, number } => {
                        church_slavonic::aorist_variants(lemma, person, number)
                    }
                    VerbCellKind::Imperative { person, number } => {
                        church_slavonic::imperative_variants(lemma, person, number)
                    }
                    VerbCellKind::LParticiple { gender, number } => {
                        church_slavonic::l_participle_variants(lemma, gender, number)
                    }
                    VerbCellKind::Infinitive => church_slavonic::infinitive_variants(lemma),
                    VerbCellKind::Supine => church_slavonic::supine_variants(lemma),
                    VerbCellKind::VerbalNoun => church_slavonic::verbal_noun_variants(lemma),
                    VerbCellKind::PresentActiveParticiple => {
                        church_slavonic::present_active_participle_variants(lemma)
                    }
                    VerbCellKind::PresentPassiveParticiple => {
                        church_slavonic::present_passive_participle_variants(lemma)
                    }
                    VerbCellKind::PastActiveParticiple => {
                        church_slavonic::past_active_participle_variants(lemma)
                    }
                    VerbCellKind::PastPassiveParticiple => {
                        church_slavonic::past_passive_participle_variants(lemma)
                    }
                };
                produced.ok().map(|variants| (kind, variants))
            })
            .collect();
        verb_cells += expected.len();
        if church_slavonic::verb_paradigm(lemma).as_ref() != Ok(&expected) && mismatches.len() < 20
        {
            mismatches.push(format!(
                "verb {lemma}: paradigm disagrees with single-cell API"
            ));
        }
    }

    // Declined-participle paradigms: for every attested verb lexeme, each of
    // the four kinds x long/short enumerations must contain exactly the
    // cells `participle_variants` serves, in order, with the same lists.
    let mut participle_cells = 0usize;
    for lemma in &verb_lemmas {
        for kind in [
            ParticipleKind::PresentActive,
            ParticipleKind::PresentPassive,
            ParticipleKind::PastActive,
            ParticipleKind::PastPassive,
        ] {
            for form in [AdjectiveForm::Long, AdjectiveForm::Short] {
                let expected: Vec<(Case, Number, Gender, Vec<String>)> = grid()
                    .flat_map(|(case, number)| {
                        Gender::ALL
                            .into_iter()
                            .map(move |gender| (case, number, gender))
                    })
                    .filter_map(|(case, number, gender)| {
                        church_slavonic::participle_variants(
                            lemma, kind, case, number, gender, form,
                        )
                        .ok()
                        .map(|variants| (case, number, gender, variants))
                    })
                    .collect();
                participle_cells += expected.len();
                if church_slavonic::participle_paradigm(lemma, kind, form).as_ref() != Ok(&expected)
                    && mismatches.len() < 20
                {
                    mismatches.push(format!(
                        "participle {lemma} ({kind:?} {form:?}): paradigm disagrees with \
                         single-cell API"
                    ));
                }
            }
        }
    }

    let closed_lemmas: BTreeSet<&String> = closed.cells.keys().map(|(lemma, _)| lemma).collect();
    let mut closed_cells = 0usize;
    for lemma in &closed_lemmas {
        let (pos_code, shape) = closed.meta[*lemma];
        let single = |case, number, gender| match pos_code {
            1 => church_slavonic::pronoun_form_variants(lemma, case, number, gender),
            2 => church_slavonic::numeral_form_variants(lemma, case, number, gender),
            _ => church_slavonic::determiner_form_variants(lemma, case, number, gender),
        };
        let expected: Vec<(Case, Number, Option<Gender>, Vec<String>)> = if shape & 2 != 0 {
            grid()
                .flat_map(|(case, number)| {
                    Gender::ALL
                        .into_iter()
                        .map(move |gender| (case, number, gender))
                })
                .filter_map(|(case, number, gender)| {
                    single(case, number, gender)
                        .ok()
                        .map(|variants| (case, number, Some(gender), variants))
                })
                .collect()
        } else if shape & 1 != 0 {
            grid()
                .filter_map(|(case, number)| {
                    single(case, number, Gender::Masculine)
                        .ok()
                        .map(|variants| (case, number, None, variants))
                })
                .collect()
        } else {
            Vec::new()
        };
        closed_cells += expected.len();
        let paradigm = match pos_code {
            1 => church_slavonic::pronoun_form_paradigm(lemma),
            2 => church_slavonic::numeral_form_paradigm(lemma),
            _ => church_slavonic::determiner_form_paradigm(lemma),
        };
        if paradigm.as_ref() != Ok(&expected) && mismatches.len() < 20 {
            mismatches.push(format!(
                "closed {lemma}: paradigm disagrees with single-cell API"
            ));
        }
    }

    println!("rewrite pilot paradigm consistency (enumerated paradigms vs the single-cell API)");
    println!(
        "  nouns: {} lexemes / {noun_cells} servable cells consistent",
        noun_lemmas.len()
    );
    println!(
        "  adjectives: {} lexemes / {adjective_cells} servable cells consistent",
        adjective_lemmas.len()
    );
    println!(
        "  verbs: {} lexemes / {verb_cells} servable cells consistent",
        verb_lemmas.len()
    );
    println!(
        "  declined participles: {} lexemes x 4 kinds x 2 forms / {participle_cells} servable \
         cells consistent",
        verb_lemmas.len()
    );
    println!(
        "  closed classes: {} lexemes / {closed_cells} servable cells consistent",
        closed_lemmas.len()
    );
    for line in &mismatches {
        println!("  MISMATCH {line}");
    }
    if !mismatches.is_empty() {
        return Err(format!(
            "paradigm consistency failed for {} lexeme(s)",
            mismatches.len()
        )
        .into());
    }
    Ok(())
}

/// Differential gate for the value-driven numeral composer: the fat
/// old-church-slavonic facade is the reference. For a deterministic sweep of
/// values (1..=100 plus representative larger values across every supported
/// order of magnitude) x case x gender, the pilot's `numeral()` must equal
/// the old `compound_cardinal` (default options) primary text for 11..=10,000
/// and the old lemma-keyed simple-cardinal path for 1..=10, and the pilot's
/// `distributive_numeral()` must equal the old `distributive_cardinal`.
/// Cells the old facade rejects must be rejected by the pilot too.
fn numeral_value_differential() -> Result<(), Box<dyn Error>> {
    fn old_cardinal_reference(value: u16, case: Case, gender: Gender) -> Option<String> {
        if value >= 11 {
            return old_church_slavonic::compound_cardinal(value, case, Some(gender))
                .or_else(|_| old_church_slavonic::compound_cardinal(value, case, None))
                .ok()
                .map(|realized| realized.primary_text());
        }
        let (lemma, number, gendered) = match value {
            1 => ("ѥдинъ", Number::Singular, true),
            2 => ("дъва", Number::Dual, true),
            3 => ("триѥ", Number::Plural, true),
            4 => ("четыре", Number::Plural, true),
            5 => ("пѧть", Number::Singular, false),
            6 => ("шесть", Number::Singular, false),
            7 => ("седмь", Number::Singular, false),
            8 => ("осмь", Number::Singular, false),
            9 => ("девѧть", Number::Singular, false),
            10 => ("десѧть", Number::Singular, false),
            _ => unreachable!("simple cardinal sweep covers 1..=10"),
        };
        let result = if gendered {
            old_church_slavonic::gendered_numeral(lemma, case, number, gender)
        } else {
            old_church_slavonic::numeral(lemma, case, number)
        };
        result.ok().map(|forms| forms.primary_text().to_string())
    }

    fn old_distributive_reference(value: u16, gender: Gender) -> Option<String> {
        old_church_slavonic::distributive_cardinal(value, Some(gender))
            .or_else(|_| old_church_slavonic::distributive_cardinal(value, None))
            .ok()
            .map(|realized| realized.primary_text())
    }

    let mut values: Vec<u16> = (1..=100).collect();
    values.extend([
        101, 110, 111, 123, 199, 200, 222, 300, 333, 400, 444, 500, 555, 600, 666, 700, 777, 800,
        888, 900, 999, 1_000, 1_001, 1_100, 1_111, 1_234, 2_000, 2_222, 3_000, 3_456, 4_000, 5_000,
        5_555, 6_000, 7_000, 8_000, 9_000, 9_999, 10_000,
    ]);

    let mut cardinal_agreements = 0usize;
    let mut cardinal_total = 0usize;
    let mut distributive_agreements = 0usize;
    let mut distributive_total = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    for &value in &values {
        for gender in Gender::ALL {
            for case in Case::ALL {
                cardinal_total += 1;
                let new =
                    church_slavonic::numeral(u64::from(value), case, gender, Animacy::Inanimate);
                let old = old_cardinal_reference(value, case, gender);
                match (&new, &old) {
                    (Ok(new_text), Some(old_text)) if new_text == old_text => {
                        cardinal_agreements += 1;
                    }
                    (Err(_), None) => cardinal_agreements += 1,
                    _ => {
                        if mismatches.len() < 20 {
                            mismatches.push(format!(
                                "cardinal {value} {case:?} {gender:?}: new {new:?} vs old {old:?}"
                            ));
                        }
                    }
                }
            }
            distributive_total += 1;
            let new =
                church_slavonic::distributive_numeral(u64::from(value), gender, Animacy::Inanimate);
            let old = old_distributive_reference(value, gender);
            match (&new, &old) {
                (Ok(new_text), Some(old_text)) if new_text == old_text => {
                    distributive_agreements += 1;
                }
                (Err(_), None) => distributive_agreements += 1,
                _ => {
                    if mismatches.len() < 20 {
                        mismatches.push(format!(
                            "distributive {value} {gender:?}: new {new:?} vs old {old:?}"
                        ));
                    }
                }
            }
        }
    }

    println!(
        "rewrite pilot numeral differential (value-driven numeral() vs the old facade \
         compound_cardinal/simple-cardinal reference, default options)"
    );
    println!(
        "  cardinals: {cardinal_agreements}/{cardinal_total} value x case x gender cells agree \
         ({} distinct values swept)",
        values.len()
    );
    println!(
        "  distributives: {distributive_agreements}/{distributive_total} value x gender cells agree"
    );
    for line in &mismatches {
        println!("  MISMATCH {line}");
    }
    if cardinal_agreements != cardinal_total || distributive_agreements != distributive_total {
        return Err(format!(
            "numeral differential {cardinal_agreements}/{cardinal_total} cardinal and \
             {distributive_agreements}/{distributive_total} distributive agreements, expected 100%"
        )
        .into());
    }
    Ok(())
}

/// Cross-POS lemma-key inventory and lexeme-id mapping consumed by the
/// dictionary generator (`cargo xtask rewrite-dictionary`): every pilot
/// lemma key with its part-of-speech code, plus the deterministic
/// extracted-lexeme-id -> lemma-key assignment the residue emitters use
/// (the same `homograph_keys` sort, so a dictionary sense that names a
/// lexeme id lands on exactly the key the facade serves that lexeme under).
pub(crate) struct DictionarySupport {
    /// Extracted lexeme id -> pilot lemma key (bare or numeric-suffixed).
    pub key_by_lexeme: BTreeMap<String, String>,
    /// Pilot lemma key -> part-of-speech code
    /// (1 noun, 2 adjective, 3 verb, 4 pronoun, 5 numeral, 6 determiner).
    pub lemma_pos: BTreeMap<String, u8>,
}

pub(crate) fn dictionary_support(root: &Path) -> Result<DictionarySupport, Box<dyn Error>> {
    let nouns = load_noun_oracle(root)?;
    let adjectives = load_adjective_oracle(root)?;
    let verbs = load_verb_oracle(root)?;
    let closed = load_closed_oracle(root)?;
    let mut key_by_lexeme = BTreeMap::new();
    let mut lemma_pos: BTreeMap<String, u8> = BTreeMap::new();
    let per_pos: [(&BTreeMap<String, String>, Vec<&String>, u8); 3] = [
        (&nouns.key_by_lexeme, nouns.meta.keys().collect(), 1),
        (
            &adjectives.key_by_lexeme,
            adjectives.meta.keys().collect(),
            2,
        ),
        (&verbs.key_by_lexeme, verbs.meta.keys().collect(), 3),
    ];
    for (oracle_keys, oracle_meta_keys, pos) in per_pos {
        for key in oracle_meta_keys {
            lemma_pos.insert(key.clone(), pos);
        }
        for (id, key) in oracle_keys {
            key_by_lexeme.insert(id.clone(), key.clone());
        }
    }
    for (lemma, (pos, _)) in &closed.meta {
        lemma_pos.insert(lemma.clone(), pos + 3);
    }
    for (id, key) in &closed.key_by_lexeme {
        key_by_lexeme.insert(id.clone(), key.clone());
    }
    Ok(DictionarySupport {
        key_by_lexeme,
        lemma_pos,
    })
}
