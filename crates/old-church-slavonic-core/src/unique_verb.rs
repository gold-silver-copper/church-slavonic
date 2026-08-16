//! Closed inventory of the nineteen unique Old Church Slavonic verb profiles.
//!
//! Polivanova 2023 §§417 and 516–605 treats these lexemes wholesale rather
//! than as members of the reusable irregular work-stem groups in §§434–440.
//! The inventory therefore lives above the productive verb rules: exact
//! present cells preserve each exceptional profile, independent principal
//! parts feed productive subsystems, and a source dash becomes an explicit
//! unreconstructable defect instead of missing metadata.

use crate::verb::{
    VerbLexeme, insert_imperative_singular, set_imperative, set_imperfect, set_l_participle,
    set_new_aorist, set_past_active, set_past_passive, set_present_active, set_present_passive,
    set_sigmatic_vowel_aorist,
};
use crate::{
    FiniteTense, FiniteVerbCell, ImperativeCell, ImperativeFormation, ImperfectFormation,
    ParticipleKind, PastActiveParticipleFormation, PastPassiveParticipleFormation,
    PresentActiveParticipleFormation, PresentPassiveParticipleFormation, VerbAspect, VerbClass,
    VerbDefectKind, VerbMorphologyCell, VerbMorphologySystem,
};

/// The source-level shape of a unique verb profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UniqueVerbProfileKind {
    Athematic,
    Suppletive,
    MixedConjugation,
    Root,
    SparseReconstruction,
}

/// Polivanova's exhaustive nineteen-member unique-verb inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UniqueVerbIdentity {
    Dati,
    Jasti,
    Vedeti,
    Imeti,
    Esmi,
    Byti,
    Hoteti,
    Dovleti,
    Iti,
    Jati,
    Stati,
    Supati,
    Vopiti,
    Sesti,
    Leshti,
    Obresti,
    Gnati,
    Pleti,
    Deti,
}

/// One lexeme in the exhaustive family union of the unique profiles.
///
/// This value records source identity only. It deliberately does not infer a
/// prefix boundary: several families have contraction, hiatus, or root
/// allomorphy that must be supplied by the later system-specific profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueVerbFamilyMember {
    profile: UniqueVerbIdentity,
    lemma: &'static str,
}

/// OSD uses natural `ы` in seven rows whose normalized grammar identities use
/// the historical `ꙑ` digraph. Keep the crosswalk closed: these are spelling
/// aliases of reviewed family members, not additional lexical paradigms.
const SOURCE_UNION_SPELLING_ALIASES: [(&str, &str); 7] = [
    ("быти", "бꙑти"),
    ("забыти", "забꙑти"),
    ("избыти", "избꙑти"),
    ("прибыти", "прибꙑти"),
    ("прѣбыти", "прѣбꙑти"),
    ("събыти", "събꙑти"),
    ("выгънати", "вꙑгънати"),
];

impl UniqueVerbFamilyMember {
    pub const COUNT: usize = 106;

    pub fn all() -> impl Iterator<Item = Self> {
        UniqueVerbIdentity::ALL.into_iter().flat_map(|profile| {
            profile
                .family_members()
                .iter()
                .copied()
                .map(move |lemma| Self { profile, lemma })
        })
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        let canonical = SOURCE_UNION_SPELLING_ALIASES
            .iter()
            .find_map(|(source, canonical)| (*source == lemma).then_some(*canonical))
            .unwrap_or(lemma);
        Self::all().find(|member| member.lemma == canonical)
    }

    pub const fn profile(self) -> UniqueVerbIdentity {
        self.profile
    }

    pub const fn canonical_lemma(self) -> &'static str {
        self.lemma
    }

    pub const fn source_section(self) -> &'static str {
        self.profile.source_section()
    }

    /// Lexical aspect after source-listed prefixation.
    pub fn aspect(self) -> VerbAspect {
        let is_profile_citation = self.lemma == self.profile.canonical_lemma();
        match self.profile {
            UniqueVerbIdentity::Imeti | UniqueVerbIdentity::Esmi | UniqueVerbIdentity::Dovleti => {
                self.profile.aspect()
            }
            UniqueVerbIdentity::Jasti
            | UniqueVerbIdentity::Vedeti
            | UniqueVerbIdentity::Hoteti
            | UniqueVerbIdentity::Iti
            | UniqueVerbIdentity::Supati
            | UniqueVerbIdentity::Vopiti
            | UniqueVerbIdentity::Gnati
            | UniqueVerbIdentity::Pleti
                if !is_profile_citation =>
            {
                VerbAspect::Perfective
            }
            UniqueVerbIdentity::Deti => VerbAspect::Perfective,
            _ => self.profile.aspect(),
        }
    }

    /// Assemble the member's complete typed profile from its source-specific
    /// allomorph mapping.
    pub fn lexeme(self) -> VerbLexeme {
        let mut lexeme = self.profile.lexeme();
        match self.profile {
            UniqueVerbIdentity::Jasti => {
                let root = self.lemma.strip_suffix("сти").unwrap_or("ꙗ");
                replace_all_initial(&mut lexeme, "ꙗ", root);
            }
            UniqueVerbIdentity::Iti => transform_iti_member(&mut lexeme, self.lemma),
            UniqueVerbIdentity::Jati => {
                let root = self.lemma.strip_suffix("ти").unwrap_or("ꙗ");
                replace_all_initial(&mut lexeme, "ꙗ", root);
            }
            UniqueVerbIdentity::Vopiti => {
                let root = self.lemma.strip_suffix("ти").unwrap_or("въпи");
                replace_all_initial(&mut lexeme, "въпи", root);
            }
            UniqueVerbIdentity::Obresti => {
                let root = match self.lemma {
                    "изобрѣсти" => "изобр",
                    "обрѣсти" => "обр",
                    "приобрѣсти" => "приобр",
                    "сърѣсти" => "сър",
                    _ => "обр",
                };
                replace_all_initial(&mut lexeme, "обр", root);
            }
            UniqueVerbIdentity::Gnati => transform_gnati_member(&mut lexeme, self.lemma),
            profile => {
                if let Some(prefix) = self.lemma.strip_suffix(profile.canonical_lemma()) {
                    prepend_all(&mut lexeme, prefix);
                }
            }
        }
        lexeme.lemma = self.lemma.to_string();
        lexeme.aspect = Some(self.aspect());
        configure_family_specific_defects(&mut lexeme, self);
        lexeme
    }
}

impl UniqueVerbIdentity {
    pub const ALL: [Self; 19] = [
        Self::Dati,
        Self::Jasti,
        Self::Vedeti,
        Self::Imeti,
        Self::Esmi,
        Self::Byti,
        Self::Hoteti,
        Self::Dovleti,
        Self::Iti,
        Self::Jati,
        Self::Stati,
        Self::Supati,
        Self::Vopiti,
        Self::Sesti,
        Self::Leshti,
        Self::Obresti,
        Self::Gnati,
        Self::Pleti,
        Self::Deti,
    ];

    /// Engine citation spelling. Source aliases retain normalized alternatives.
    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::Dati => "дати",
            Self::Jasti => "ꙗсти",
            Self::Vedeti => "вѣдѣти",
            Self::Imeti => "имѣти",
            Self::Esmi => "ѥсмь",
            Self::Byti => "бꙑти",
            Self::Hoteti => "хотѣти",
            Self::Dovleti => "довьлѣти",
            Self::Iti => "ити",
            Self::Jati => "ꙗти",
            Self::Stati => "стати",
            Self::Supati => "съпати",
            Self::Vopiti => "въпити",
            Self::Sesti => "сѣсти",
            Self::Leshti => "лещи",
            Self::Obresti => "обрѣсти",
            Self::Gnati => "гънати",
            Self::Pleti => "плѣти",
            Self::Deti => "дѣти",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::Dati => &["дати"],
            Self::Jasti => &["ꙗсти", "ѣсти"],
            Self::Vedeti => &["вѣдѣти"],
            Self::Imeti => &["имѣти"],
            Self::Esmi => &["ѥсмь", "есмь"],
            Self::Byti => &["бꙑти", "быти"],
            Self::Hoteti => &["хотѣти"],
            Self::Dovleti => &["довьлѣти"],
            Self::Iti => &["ити"],
            Self::Jati => &["ꙗти"],
            Self::Stati => &["стати"],
            Self::Supati => &["съпати"],
            Self::Vopiti => &["въпити"],
            Self::Sesti => &["сѣсти"],
            Self::Leshti => &["лещи"],
            Self::Obresti => &["обрѣсти"],
            Self::Gnati => &["гънати"],
            Self::Pleti => &["плѣти"],
            Self::Deti => &["дѣти"],
        }
    }

    /// Exact family membership from Polivanova §§520–604, in source order.
    ///
    /// The `дѣти` profile is intentionally represented only by its five
    /// prefixed lexemes: §604 and its note exclude a prefixless dictionary
    /// headword even though §602 uses `дѣти` as the grammatical profile label.
    pub const fn family_members(self) -> &'static [&'static str] {
        match self {
            Self::Dati => &[
                "дати",
                "въдати",
                "въздати",
                "издати",
                "отъдати",
                "подати",
                "придати",
                "продати",
                "прѣдати",
            ],
            Self::Jasti => &["ꙗсти", "изѣсти", "обѣсти", "поꙗсти", "сънѣсти"],
            Self::Vedeti => &[
                "вѣдѣти",
                "заповѣдѣти",
                "извѣдѣти",
                "исповѣдѣти",
                "навѣдѣти",
                "недовѣдѣти",
                "повѣдѣти",
                "проповѣдѣти",
                "проувѣдѣти",
                "съвѣдѣти",
                "съповѣдѣти",
                "увѣдѣти",
            ],
            Self::Imeti => &["имѣти", "недоимѣти"],
            Self::Esmi => &["ѥсмь"],
            Self::Byti => &["бꙑти", "забꙑти", "избꙑти", "прибꙑти", "прѣбꙑти", "събꙑти"],
            Self::Hoteti => &["хотѣти", "въсхотѣти", "похотѣти"],
            Self::Dovleti => &["довьлѣти"],
            Self::Iti => &[
                "ити",
                "възити",
                "вънити",
                "доити",
                "заити",
                "изити",
                "наити",
                "низъити",
                "обити",
                "отити",
                "подъити",
                "поити",
                "прити",
                "проити",
                "прѣвъзити",
                "прѣдъити",
                "прѣити",
                "разити",
                "сънити",
            ],
            Self::Jati => &["ꙗти", "възѣти", "въꙗти", "приꙗти", "прѣꙗти"],
            Self::Stati => &[
                "стати",
                "въстати",
                "достати",
                "настати",
                "остати",
                "пристати",
                "прѣдъстати",
                "прѣстати",
                "състати",
                "устати",
            ],
            Self::Supati => &["съпати", "посъпати"],
            Self::Vopiti => &["въпити", "възъпити"],
            Self::Sesti => &[
                "сѣсти",
                "въсѣсти",
                "осѣсти",
                "посѣсти",
                "просѣсти",
                "прѣдъсѣсти",
                "съсѣсти",
            ],
            Self::Leshti => &["лещи", "възлещи", "облещи", "улещи"],
            Self::Obresti => &["изобрѣсти", "обрѣсти", "приобрѣсти", "сърѣсти"],
            Self::Gnati => &[
                "гънати",
                "вꙑгънати",
                "изгънати",
                "отъгънати",
                "погънати",
                "прогънати",
                "разгънати",
            ],
            Self::Pleti => &["плѣти", "исплѣти"],
            Self::Deti => &["въдѣти", "въздѣти", "задѣти", "одѣти", "придѣти"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn profile_kind(self) -> UniqueVerbProfileKind {
        match self {
            Self::Dati | Self::Jasti | Self::Vedeti | Self::Imeti => {
                UniqueVerbProfileKind::Athematic
            }
            Self::Esmi | Self::Byti | Self::Iti => UniqueVerbProfileKind::Suppletive,
            Self::Hoteti | Self::Dovleti | Self::Supati | Self::Vopiti => {
                UniqueVerbProfileKind::MixedConjugation
            }
            Self::Pleti => UniqueVerbProfileKind::SparseReconstruction,
            Self::Jati
            | Self::Stati
            | Self::Sesti
            | Self::Leshti
            | Self::Obresti
            | Self::Gnati
            | Self::Deti => UniqueVerbProfileKind::Root,
        }
    }

    pub const fn aspect(self) -> VerbAspect {
        match self {
            Self::Dati
            | Self::Byti
            | Self::Jati
            | Self::Stati
            | Self::Sesti
            | Self::Leshti
            | Self::Obresti => VerbAspect::Perfective,
            Self::Jasti | Self::Iti => VerbAspect::Biaspectual,
            Self::Vedeti
            | Self::Imeti
            | Self::Esmi
            | Self::Hoteti
            | Self::Dovleti
            | Self::Supati
            | Self::Vopiti
            | Self::Gnati
            | Self::Pleti
            | Self::Deti => VerbAspect::Imperfective,
        }
    }

    /// Polivanova section containing the lexeme profile.
    pub const fn source_section(self) -> &'static str {
        match self {
            Self::Dati => "§§517–521",
            Self::Jasti => "§§522–526",
            Self::Vedeti => "§§527–531",
            Self::Imeti => "§§532–537",
            Self::Esmi => "§§538–542",
            Self::Byti => "§§543–549",
            Self::Hoteti => "§§550–555",
            Self::Dovleti => "§§556–560",
            Self::Iti => "§§561–564",
            Self::Jati => "§§565–569",
            Self::Stati => "§§570–573",
            Self::Supati => "§§574–577",
            Self::Vopiti => "§§578–581",
            Self::Sesti => "§§582–585",
            Self::Leshti => "§§586–589",
            Self::Obresti => "§§590–593",
            Self::Gnati => "§§594–597",
            Self::Pleti => "§§598–601",
            Self::Deti => "§§602–605",
        }
    }

    /// Source-reviewed nine-cell present profile, ordered by number then person.
    pub const fn present_paradigm(self) -> [&'static str; 9] {
        match self {
            Self::Dati => [
                "дамь",
                "даси",
                "дастъ",
                "давѣ",
                "даста",
                "дасте",
                "дамъ",
                "дасте",
                "дадѧтъ",
            ],
            Self::Jasti => [
                "ꙗмь",
                "ꙗси",
                "ꙗстъ",
                "ꙗвѣ",
                "ꙗста",
                "ꙗсте",
                "ꙗмъ",
                "ꙗсте",
                "ꙗдѧтъ",
            ],
            Self::Vedeti => [
                "вѣмь",
                "вѣси",
                "вѣстъ",
                "вѣвѣ",
                "вѣста",
                "вѣсте",
                "вѣмъ",
                "вѣсте",
                "вѣдѧтъ",
            ],
            Self::Imeti => [
                "имамь",
                "имаши",
                "иматъ",
                "имавѣ",
                "имата",
                "имате",
                "имамъ",
                "имате",
                "имѫтъ",
            ],
            Self::Esmi => [
                "ѥсмь", "ѥси", "ѥстъ", "ѥсвѣ", "ѥста", "ѥсте", "ѥсмъ", "ѥсте", "сѫтъ",
            ],
            Self::Byti => [
                "бѫдѫ",
                "бѫдеши",
                "бѫдетъ",
                "бѫдевѣ",
                "бѫдета",
                "бѫдете",
                "бѫдемъ",
                "бѫдете",
                "бѫдѫтъ",
            ],
            Self::Hoteti => [
                "хощѫ",
                "хощеши",
                "хощетъ",
                "хощевѣ",
                "хощета",
                "хощете",
                "хощемъ",
                "хощете",
                "хотѧтъ",
            ],
            Self::Dovleti => [
                "довьлѭ",
                "довьлѥши",
                "довьлѥтъ",
                "довьлѥвѣ",
                "довьлѥта",
                "довьлѥте",
                "довьлѥмъ",
                "довьлѥте",
                "довьлѫтъ",
            ],
            Self::Iti => [
                "идѫ",
                "идеши",
                "идетъ",
                "идевѣ",
                "идета",
                "идете",
                "идемъ",
                "идете",
                "идѫтъ",
            ],
            Self::Jati => [
                "ꙗдѫ",
                "ꙗдеши",
                "ꙗдетъ",
                "ꙗдевѣ",
                "ꙗдета",
                "ꙗдете",
                "ꙗдемъ",
                "ꙗдете",
                "ꙗдѫтъ",
            ],
            Self::Stati => [
                "станѫ",
                "станеши",
                "станетъ",
                "станевѣ",
                "станета",
                "станете",
                "станемъ",
                "станете",
                "станѫтъ",
            ],
            Self::Supati => [
                "съплѭ",
                "съпиши",
                "съпитъ",
                "съпивѣ",
                "съпита",
                "съпите",
                "съпимъ",
                "съпите",
                "съпѧтъ",
            ],
            Self::Vopiti => [
                "въпиѭ",
                "въпиѥши",
                "въпиѥтъ",
                "въпиѥвѣ",
                "въпиѥта",
                "въпиѥте",
                "въпиѥмъ",
                "въпиѥте",
                "въпиѭтъ",
            ],
            Self::Sesti => [
                "сѧдѫ",
                "сѧдеши",
                "сѧдетъ",
                "сѧдевѣ",
                "сѧдета",
                "сѧдете",
                "сѧдемъ",
                "сѧдете",
                "сѧдѫтъ",
            ],
            Self::Leshti => [
                "лѧгѫ",
                "лѧжеши",
                "лѧжетъ",
                "лѧжевѣ",
                "лѧжета",
                "лѧжете",
                "лѧжемъ",
                "лѧжете",
                "лѧгѫтъ",
            ],
            Self::Obresti => [
                "обрѧщѫ",
                "обрѧщеши",
                "обрѧщетъ",
                "обрѧщевѣ",
                "обрѧщета",
                "обрѧщете",
                "обрѧщемъ",
                "обрѧщете",
                "обрѧщѫтъ",
            ],
            Self::Gnati => [
                "женѫ",
                "женеши",
                "женетъ",
                "женевѣ",
                "женета",
                "женете",
                "женемъ",
                "женете",
                "женѫтъ",
            ],
            Self::Pleti => [
                "плѣвѫ",
                "плѣвеши",
                "плѣветъ",
                "плѣвевѣ",
                "плѣвета",
                "плѣвете",
                "плѣвемъ",
                "плѣвете",
                "плѣвѫтъ",
            ],
            Self::Deti => [
                "дежѫ",
                "дежеши",
                "дежетъ",
                "дежевѣ",
                "дежета",
                "дежете",
                "дежемъ",
                "дежете",
                "дежѫтъ",
            ],
        }
    }

    /// Assemble a complete typed rule input for the profile.
    pub fn lexeme(self) -> VerbLexeme {
        let mut lexeme = VerbLexeme::new(self.canonical_lemma(), VerbClass::Irregular);
        lexeme.aspect = Some(self.aspect());
        insert_present(&mut lexeme, self.present_paradigm());

        match self {
            Self::Dati => {
                set_imperfect(&mut lexeme, "дад", ImperfectFormation::PresentYatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "да", "да");
                insert_athematic_imperative(&mut lexeme, "дажь", "дад");
                set_l_participle(&mut lexeme, "да");
                set_present_active(
                    &mut lexeme,
                    "дад",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "да", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "да", PastPassiveParticipleFormation::N);
            }
            Self::Jasti => {
                set_imperfect(&mut lexeme, "ꙗд", ImperfectFormation::PresentYatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "ꙗ", "ꙗ");
                insert_athematic_imperative(&mut lexeme, "ꙗжь", "ꙗд");
                set_l_participle(&mut lexeme, "ꙗ");
                set_present_active(
                    &mut lexeme,
                    "ꙗд",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "ꙗд", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "ꙗд", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "ꙗд", PastPassiveParticipleFormation::En);
            }
            Self::Vedeti => {
                set_imperfect(&mut lexeme, "вѣдѣ", ImperfectFormation::A);
                set_sigmatic_vowel_aorist(&mut lexeme, "вѣдѣ", "вѣдѣ");
                insert_athematic_imperative(&mut lexeme, "вѣжь", "вѣд");
                set_l_participle(&mut lexeme, "вѣдѣ");
                set_present_active(
                    &mut lexeme,
                    "вѣд",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "вѣд", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "вѣдѣ", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "вѣдѣ", PastPassiveParticipleFormation::N);
            }
            Self::Imeti => {
                set_imperfect(&mut lexeme, "имѣ", ImperfectFormation::A);
                set_sigmatic_vowel_aorist(&mut lexeme, "имѣ", "имѣ");
                set_imperative(&mut lexeme, "имѣ", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "имѣ");
                set_present_active(
                    &mut lexeme,
                    "им",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "имѣ", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "имѣ", PastPassiveParticipleFormation::N);
            }
            Self::Esmi => configure_defective_esmi(&mut lexeme),
            Self::Byti => {
                set_imperfect(&mut lexeme, "б", ImperfectFormation::PresentYatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "бꙑ", "бꙑ");
                set_imperative(&mut lexeme, "бѫд", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "бꙑ");
                set_present_active(
                    &mut lexeme,
                    "бѫд",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "бꙑ", PastActiveParticipleFormation::Vush);
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
                );
            }
            Self::Hoteti => {
                set_imperfect(&mut lexeme, "хотѣ", ImperfectFormation::A);
                set_sigmatic_vowel_aorist(&mut lexeme, "хотѣ", "хотѣ");
                set_imperative(&mut lexeme, "хощ", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "хотѣ");
                set_present_active(
                    &mut lexeme,
                    "хот",
                    PresentActiveParticipleFormation::YeshtSoft,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "хотѣ", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "хотѣ", PastPassiveParticipleFormation::N);
            }
            Self::Dovleti => {
                set_imperfect(&mut lexeme, "довьлѣ", ImperfectFormation::A);
                set_sigmatic_vowel_aorist(&mut lexeme, "довьлѣ", "довьлѣ");
                unreconstructable(&mut lexeme, VerbMorphologySystem::Imperative);
                set_l_participle(&mut lexeme, "довьлѣ");
                set_present_active(
                    &mut lexeme,
                    "довьлѣ",
                    PresentActiveParticipleFormation::IotatedYushtSoft,
                );
                for kind in [
                    ParticipleKind::PresentPassive,
                    ParticipleKind::PastActive,
                    ParticipleKind::PastPassive,
                ] {
                    unreconstructable(&mut lexeme, VerbMorphologySystem::Participle(kind));
                }
            }
            Self::Iti => {
                set_imperfect(&mut lexeme, "ид", ImperfectFormation::PresentYatA);
                set_new_aorist(&mut lexeme, "ид");
                set_imperative(&mut lexeme, "ид", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "шь");
                set_present_active(
                    &mut lexeme,
                    "ид",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "ид", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "шьд", PastActiveParticipleFormation::Ush);
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
                );
            }
            Self::Jati => {
                set_imperfect(&mut lexeme, "ꙗд", ImperfectFormation::PresentYatA);
                set_new_aorist(&mut lexeme, "ꙗд");
                set_imperative(&mut lexeme, "ꙗд", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "ꙗ");
                set_present_active(
                    &mut lexeme,
                    "ꙗд",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "ꙗ", PastActiveParticipleFormation::Vush);
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
                );
            }
            Self::Stati => {
                set_imperfect(&mut lexeme, "стан", ImperfectFormation::PresentYatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "ста", "ста");
                set_imperative(&mut lexeme, "стан", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "ста");
                set_present_active(
                    &mut lexeme,
                    "стан",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "ста", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "ста", PastPassiveParticipleFormation::N);
            }
            Self::Supati => {
                set_imperfect(&mut lexeme, "съпа", ImperfectFormation::A);
                set_sigmatic_vowel_aorist(&mut lexeme, "съпа", "съпа");
                set_imperative(&mut lexeme, "съп", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "съпа");
                set_present_active(
                    &mut lexeme,
                    "съп",
                    PresentActiveParticipleFormation::YeshtSoft,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "съпа", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "съпа", PastPassiveParticipleFormation::N);
            }
            Self::Vopiti => {
                set_imperfect(&mut lexeme, "въпи", ImperfectFormation::YatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "въпи", "въпи");
                set_imperative(&mut lexeme, "въпи", ImperativeFormation::ISeries);
                insert_imperative_singular(&mut lexeme, "въпи");
                set_l_participle(&mut lexeme, "въпи");
                set_present_active(
                    &mut lexeme,
                    "въпи",
                    PresentActiveParticipleFormation::IotatedYushtSoft,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "въпи", PastActiveParticipleFormation::Vush);
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
                );
            }
            Self::Sesti => {
                set_imperfect(&mut lexeme, "сѣд", ImperfectFormation::YatA);
                set_new_aorist(&mut lexeme, "сѣд");
                set_imperative(&mut lexeme, "сѧд", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "сѣ");
                set_present_active(
                    &mut lexeme,
                    "сѧд",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "сѣд", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "сѣд", PastPassiveParticipleFormation::En);
            }
            Self::Leshti => {
                lexeme
                    .exact_forms
                    .insert(VerbMorphologyCell::Infinitive, "лещи".to_string());
                lexeme
                    .exact_forms
                    .insert(VerbMorphologyCell::Supine, "лещь".to_string());
                set_imperfect(&mut lexeme, "леж", ImperfectFormation::PresentYatA);
                set_new_aorist(&mut lexeme, "лег");
                set_imperative(&mut lexeme, "лѧз", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "лег");
                set_present_active(
                    &mut lexeme,
                    "лѧг",
                    PresentActiveParticipleFormation::YushtHard,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "лег", PastActiveParticipleFormation::Ush);
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
                );
            }
            Self::Obresti => {
                set_imperfect(&mut lexeme, "обрѧщ", ImperfectFormation::PresentYatA);
                set_new_aorist(&mut lexeme, "обрѣт");
                set_imperative(&mut lexeme, "обрѧщ", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "обрѣ");
                set_present_active(
                    &mut lexeme,
                    "обрѧщ",
                    PresentActiveParticipleFormation::MixedYushtSoft,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "обрѣт", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "обрѣт", PastPassiveParticipleFormation::En);
            }
            Self::Gnati => {
                set_imperfect(&mut lexeme, "жен", ImperfectFormation::PresentYatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "гъна", "гъна");
                set_imperative(&mut lexeme, "жен", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "гъна");
                set_present_active(
                    &mut lexeme,
                    "жен",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "жен", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "гъна", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "гъна", PastPassiveParticipleFormation::N);
            }
            Self::Pleti => configure_sparse_pleti(&mut lexeme),
            Self::Deti => {
                set_imperfect(&mut lexeme, "дѣ", ImperfectFormation::YatA);
                set_sigmatic_vowel_aorist(&mut lexeme, "дѣ", "дѣ");
                set_imperative(&mut lexeme, "деж", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "дѣ");
                set_present_active(
                    &mut lexeme,
                    "деж",
                    PresentActiveParticipleFormation::MixedYushtSoft,
                );
                unreconstructable(
                    &mut lexeme,
                    VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
                );
                set_past_active(&mut lexeme, "дѣ", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "дѣ", PastPassiveParticipleFormation::N);
            }
        }
        lexeme
    }
}

fn insert_present(lexeme: &mut VerbLexeme, forms: [&str; 9]) {
    for (cell, form) in FiniteVerbCell::for_tense(FiniteTense::Present).zip(forms) {
        lexeme
            .exact_forms
            .insert(VerbMorphologyCell::Finite(cell), form.to_string());
    }
}

fn map_all_surface_values(lexeme: &mut VerbLexeme, mut map: impl FnMut(&str) -> String) {
    for form in lexeme.exact_forms.values_mut() {
        *form = map(form);
    }
    for value in [
        &mut lexeme.stems.present,
        &mut lexeme.stems.present_first_singular,
        &mut lexeme.stems.imperfect,
        &mut lexeme.stems.aorist,
        &mut lexeme.stems.aorist_second_third_singular,
        &mut lexeme.stems.imperative,
        &mut lexeme.stems.l_participle,
        &mut lexeme.stems.present_active_participle,
        &mut lexeme.stems.present_passive_participle,
        &mut lexeme.stems.past_active_participle,
        &mut lexeme.stems.past_passive_participle,
    ]
    .into_iter()
    .flatten()
    {
        *value = map(value);
    }
}

fn prepend_all(lexeme: &mut VerbLexeme, prefix: &str) {
    if prefix.is_empty() {
        return;
    }
    map_all_surface_values(lexeme, |value| format!("{prefix}{value}"));
}

fn replace_all_initial(lexeme: &mut VerbLexeme, from: &str, to: &str) {
    if from == to {
        return;
    }
    map_all_surface_values(lexeme, |value| {
        value
            .strip_prefix(from)
            .map_or_else(|| value.to_string(), |suffix| format!("{to}{suffix}"))
    });
}

fn transform_iti_member(lexeme: &mut VerbLexeme, lemma: &str) {
    let finite_prefix = lemma.strip_suffix("ити").unwrap_or("");
    let finite_root = format!("{finite_prefix}ид");
    for form in lexeme.exact_forms.values_mut() {
        if let Some(suffix) = form.strip_prefix("ид") {
            *form = format!("{finite_root}{suffix}");
        }
    }
    for stem in [
        &mut lexeme.stems.imperfect,
        &mut lexeme.stems.aorist,
        &mut lexeme.stems.imperative,
        &mut lexeme.stems.present_active_participle,
        &mut lexeme.stems.present_passive_participle,
    ] {
        if stem.as_deref().is_some_and(|value| value.starts_with("ид")) {
            *stem = Some(finite_root.clone());
        }
    }

    let suppletive_prefix = match lemma {
        "ити" => "",
        "възити" => "въз",
        "вънити" => "въ",
        "доити" => "до",
        "заити" => "за",
        "изити" => "и",
        "наити" => "на",
        "низъити" => "низъ",
        "обити" => "об",
        "отити" => "от",
        "подъити" => "подъ",
        "поити" => "по",
        "прити" => "при",
        "проити" => "про",
        "прѣвъзити" => "прѣвъз",
        "прѣдъити" => "прѣдъ",
        "прѣити" => "прѣ",
        "разити" => "раз",
        "сънити" => "съ",
        _ => "",
    };
    lexeme.stems.l_participle = Some(format!("{suppletive_prefix}шь"));
    lexeme.stems.past_active_participle = Some(format!("{suppletive_prefix}шьд"));
}

fn transform_gnati_member(lexeme: &mut VerbLexeme, lemma: &str) {
    let nonpresent_root = lemma.strip_suffix("ти").unwrap_or("гъна");
    let present_root = match lemma {
        "гънати" => "жен",
        "вꙑгънати" => "вꙑжен",
        "изгънати" => "ижен",
        "отъгънати" => "отъжен",
        "погънати" => "пожен",
        "прогънати" => "прожен",
        "разгънати" => "ражен",
        _ => "жен",
    };
    for form in lexeme.exact_forms.values_mut() {
        if let Some(suffix) = form.strip_prefix("жен") {
            *form = format!("{present_root}{suffix}");
        }
    }
    for stem in [
        &mut lexeme.stems.imperfect,
        &mut lexeme.stems.imperative,
        &mut lexeme.stems.present_active_participle,
        &mut lexeme.stems.present_passive_participle,
    ] {
        if stem.is_some() {
            *stem = Some(present_root.to_string());
        }
    }
    for stem in [
        &mut lexeme.stems.aorist,
        &mut lexeme.stems.aorist_second_third_singular,
        &mut lexeme.stems.l_participle,
        &mut lexeme.stems.past_active_participle,
        &mut lexeme.stems.past_passive_participle,
    ] {
        if stem.is_some() {
            *stem = Some(nonpresent_root.to_string());
        }
    }
}

fn configure_family_specific_defects(lexeme: &mut VerbLexeme, member: UniqueVerbFamilyMember) {
    if member.profile == UniqueVerbIdentity::Byti
        && member.lemma != UniqueVerbIdentity::Byti.canonical_lemma()
    {
        lexeme.defective_systems.insert(
            VerbMorphologySystem::Finite(FiniteTense::Imperfect),
            VerbDefectKind::UnattestedUnreconstructable,
        );
    }
    if member.lemma == "забꙑти" {
        lexeme
            .defective_systems
            .remove(&VerbMorphologySystem::Participle(
                ParticipleKind::PastPassive,
            ));
        set_past_passive(lexeme, "забъв", PastPassiveParticipleFormation::En);
    }
}

fn insert_athematic_imperative(lexeme: &mut VerbLexeme, singular: &str, plural_stem: &str) {
    let forms = [
        singular.to_string(),
        singular.to_string(),
        format!("{plural_stem}ивѣ"),
        format!("{plural_stem}ита"),
        format!("{plural_stem}имъ"),
        format!("{plural_stem}ите"),
    ];
    for (cell, form) in ImperativeCell::SUPPORTED.into_iter().zip(forms) {
        lexeme
            .exact_forms
            .insert(VerbMorphologyCell::Imperative(cell), form);
    }
}

fn unreconstructable(lexeme: &mut VerbLexeme, system: VerbMorphologySystem) {
    lexeme
        .defective_systems
        .insert(system, VerbDefectKind::UnattestedUnreconstructable);
}

fn configure_defective_esmi(lexeme: &mut VerbLexeme) {
    set_present_active(lexeme, "с", PresentActiveParticipleFormation::YushtHard);
    for system in [
        VerbMorphologySystem::Finite(FiniteTense::Imperfect),
        VerbMorphologySystem::Finite(FiniteTense::Aorist),
        VerbMorphologySystem::Imperative,
        VerbMorphologySystem::Infinitive,
        VerbMorphologySystem::Supine,
        VerbMorphologySystem::LParticiple,
        VerbMorphologySystem::Participle(ParticipleKind::PresentPassive),
        VerbMorphologySystem::Participle(ParticipleKind::PastActive),
        VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
    ] {
        lexeme
            .defective_systems
            .insert(system, VerbDefectKind::HistoricallyInvalid);
    }
}

fn configure_sparse_pleti(lexeme: &mut VerbLexeme) {
    // Polivanova §598 directly attests only 1pl present and the present passive
    // participle. LOVe independently supplies the comparative reconstruction
    // plěv-/plě-/plěvi; the still unsupported systems remain explicit gaps.
    set_imperative(lexeme, "плѣв", ImperativeFormation::ISeries);
    set_sigmatic_vowel_aorist(lexeme, "плѣ", "плѣ");
    set_present_active(lexeme, "плѣв", PresentActiveParticipleFormation::YushtHard);
    set_present_passive(lexeme, "плѣв", PresentPassiveParticipleFormation::Om);
    for system in [
        VerbMorphologySystem::Finite(FiniteTense::Imperfect),
        VerbMorphologySystem::LParticiple,
        VerbMorphologySystem::Participle(ParticipleKind::PastActive),
        VerbMorphologySystem::Participle(ParticipleKind::PastPassive),
    ] {
        unreconstructable(lexeme, system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verb::{finite, imperative, infinitive, l_participle, participle, supine};
    use crate::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, Gender, InflectionError, LParticipleCell,
        Number, ParticipleCell, Person,
    };
    use std::collections::BTreeSet;

    fn is_explicit_defect(error: &InflectionError) -> bool {
        matches!(
            error,
            InflectionError::HistoricallyInvalidCell { .. }
                | InflectionError::UnattestedUnreconstructableCell { .. }
        )
    }

    #[test]
    fn inventory_is_exhaustive_unique_and_source_addressable() {
        assert_eq!(UniqueVerbIdentity::ALL.len(), 19);
        let lemmas = UniqueVerbIdentity::ALL
            .into_iter()
            .map(UniqueVerbIdentity::canonical_lemma)
            .collect::<BTreeSet<_>>();
        assert_eq!(lemmas.len(), 19);
        for identity in UniqueVerbIdentity::ALL {
            assert_eq!(
                UniqueVerbIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            assert!(identity.source_section().starts_with('§'));
        }
        assert_eq!(
            UniqueVerbIdentity::classify_source_union_lemma("ѣсти"),
            Some(UniqueVerbIdentity::Jasti)
        );
        assert_eq!(
            UniqueVerbIdentity::classify_source_union_lemma("быти"),
            Some(UniqueVerbIdentity::Byti)
        );
    }

    #[test]
    fn family_union_has_exact_source_counts_order_and_no_overlap() {
        let expected_counts = [9, 5, 12, 2, 1, 6, 3, 1, 19, 5, 10, 2, 2, 7, 4, 4, 7, 2, 5];
        for (identity, expected) in UniqueVerbIdentity::ALL.into_iter().zip(expected_counts) {
            assert_eq!(identity.family_members().len(), expected, "{identity:?}");
        }

        let members = UniqueVerbFamilyMember::all().collect::<Vec<_>>();
        assert_eq!(members.len(), UniqueVerbFamilyMember::COUNT);
        let lemmas = members
            .iter()
            .map(|member| member.canonical_lemma())
            .collect::<BTreeSet<_>>();
        assert_eq!(lemmas.len(), UniqueVerbFamilyMember::COUNT);
        for member in members {
            assert_eq!(
                UniqueVerbFamilyMember::classify_source_union_lemma(member.canonical_lemma()),
                Some(member)
            );
            crate::Lemma::parse(member.canonical_lemma()).expect("source family lemma");
        }

        assert_eq!(
            UniqueVerbIdentity::Obresti.family_members(),
            &["изобрѣсти", "обрѣсти", "приобрѣсти", "сърѣсти"]
        );
        assert!(!UniqueVerbIdentity::Deti.family_members().contains(&"дѣти"));
        for excluded_near_neighbor in ["даꙗти", "ꙗхати", "стоꙗти", "гонити", "дѣꙗти"]
        {
            assert_eq!(
                UniqueVerbFamilyMember::classify_source_union_lemma(excluded_near_neighbor),
                None,
                "{excluded_near_neighbor} belongs to a different productive lexeme"
            );
        }
    }

    #[test]
    fn source_union_natural_yeri_aliases_are_closed_and_identity_preserving() {
        assert_eq!(SOURCE_UNION_SPELLING_ALIASES.len(), 7);
        for (source, canonical) in SOURCE_UNION_SPELLING_ALIASES {
            let aliased = UniqueVerbFamilyMember::classify_source_union_lemma(source)
                .unwrap_or_else(|| panic!("missing OSD spelling {source}"));
            let direct = UniqueVerbFamilyMember::classify_source_union_lemma(canonical)
                .unwrap_or_else(|| panic!("missing canonical spelling {canonical}"));
            assert_eq!(aliased, direct, "{source} -> {canonical}");
            assert_eq!(aliased.canonical_lemma(), canonical);
        }
        assert_eq!(
            UniqueVerbFamilyMember::classify_source_union_lemma("вызгънати"),
            None,
            "the spelling crosswalk must not become a fuzzy yeri normalizer"
        );
    }

    #[test]
    fn all_nineteen_source_present_goldens_are_exact() {
        for identity in UniqueVerbIdentity::ALL {
            let lexeme = identity.lexeme();
            let actual = FiniteVerbCell::for_tense(FiniteTense::Present)
                .map(|cell| finite(&lexeme, cell).expect("unique present cell").text)
                .collect::<Vec<_>>();
            assert_eq!(actual, identity.present_paradigm(), "{identity:?}");
        }
    }

    #[test]
    fn every_profile_cell_is_realized_or_explicitly_defected() {
        for identity in UniqueVerbIdentity::ALL {
            let lexeme = identity.lexeme();
            for cell in FiniteVerbCell::all() {
                if let Err(error) = finite(&lexeme, cell) {
                    assert!(
                        is_explicit_defect(&error),
                        "{identity:?} {cell:?}: {error:?}"
                    );
                }
            }
            for cell in ImperativeCell::SUPPORTED {
                if let Err(error) = imperative(&lexeme, cell) {
                    assert!(
                        is_explicit_defect(&error),
                        "{identity:?} {cell:?}: {error:?}"
                    );
                }
            }
            for result in [infinitive(&lexeme), supine(&lexeme)] {
                if let Err(error) = result {
                    assert!(is_explicit_defect(&error), "{identity:?}: {error:?}");
                }
            }
            for cell in LParticipleCell::all() {
                if let Err(error) = l_participle(&lexeme, cell) {
                    assert!(
                        is_explicit_defect(&error),
                        "{identity:?} {cell:?}: {error:?}"
                    );
                }
            }
            for kind in ParticipleKind::ALL {
                for cell in ParticipleCell::for_kind(kind) {
                    if let Err(error) = participle(&lexeme, cell) {
                        assert!(
                            is_explicit_defect(&error),
                            "{identity:?} {cell:?}: {error:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn every_source_family_cell_is_realized_or_explicitly_defected() {
        for member in UniqueVerbFamilyMember::all() {
            let lexeme = member.lexeme();
            assert_eq!(lexeme.lemma, member.canonical_lemma(), "{member:?}");
            assert_eq!(lexeme.aspect, Some(member.aspect()), "{member:?}");

            for cell in FiniteVerbCell::all() {
                if let Err(error) = finite(&lexeme, cell) {
                    assert!(is_explicit_defect(&error), "{member:?} {cell:?}: {error:?}");
                }
            }
            for cell in ImperativeCell::SUPPORTED {
                if let Err(error) = imperative(&lexeme, cell) {
                    assert!(is_explicit_defect(&error), "{member:?} {cell:?}: {error:?}");
                }
            }
            for result in [infinitive(&lexeme), supine(&lexeme)] {
                if let Err(error) = result {
                    assert!(is_explicit_defect(&error), "{member:?}: {error:?}");
                }
            }
            for cell in LParticipleCell::all() {
                if let Err(error) = l_participle(&lexeme, cell) {
                    assert!(is_explicit_defect(&error), "{member:?} {cell:?}: {error:?}");
                }
            }
            for kind in ParticipleKind::ALL {
                for cell in ParticipleCell::for_kind(kind) {
                    if let Err(error) = participle(&lexeme, cell) {
                        assert!(is_explicit_defect(&error), "{member:?} {cell:?}: {error:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn family_allomorphs_and_source_specific_defects_are_exact() {
        let present_first_singular = FiniteVerbCell {
            tense: FiniteTense::Present,
            person: Person::First,
            number: Number::Singular,
        };
        let aorist_first_singular = FiniteVerbCell {
            tense: FiniteTense::Aorist,
            ..present_first_singular
        };
        let imperfect_first_singular = FiniteVerbCell {
            tense: FiniteTense::Imperfect,
            ..present_first_singular
        };
        let masculine_singular = LParticipleCell {
            gender: Gender::Masculine,
            number: Number::Singular,
        };
        let member = |lemma| {
            UniqueVerbFamilyMember::classify_source_union_lemma(lemma)
                .unwrap_or_else(|| panic!("source family member {lemma}"))
                .lexeme()
        };

        for (lemma, expected) in [
            ("изѣсти", "изѣмь"),
            ("възѣти", "възѣдѫ"),
            ("прити", "придѫ"),
            ("възъпити", "възъпиѭ"),
            ("сърѣсти", "сърѧщѫ"),
            ("изгънати", "иженѫ"),
            ("разгънати", "раженѫ"),
            ("одѣти", "одежѫ"),
        ] {
            assert_eq!(
                finite(&member(lemma), present_first_singular)
                    .expect("family present")
                    .text,
                expected,
                "{lemma}"
            );
        }
        for (lemma, expected) in [("прити", "пришьлъ"), ("вънити", "въшьлъ")]
        {
            assert_eq!(
                l_participle(&member(lemma), masculine_singular)
                    .expect("suppletive family l-participle")
                    .text,
                expected,
                "{lemma}"
            );
        }
        for (lemma, expected) in [("сърѣсти", "сърѣтохъ"), ("изгънати", "изгънахъ")]
        {
            assert_eq!(
                finite(&member(lemma), aorist_first_singular)
                    .expect("family aorist")
                    .text,
                expected,
                "{lemma}"
            );
        }
        assert_eq!(
            l_participle(&member("сърѣсти"), masculine_singular)
                .expect("root family l-participle")
                .text,
            "сърѣлъ"
        );

        let past_passive_nominative = ParticipleCell {
            kind: ParticipleKind::PastPassive,
            adjective: AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        };
        assert_eq!(
            participle(&member("забꙑти"), past_passive_nominative)
                .expect("attested prefixed past passive")
                .text,
            "забъвенъ"
        );
        assert!(matches!(
            finite(&member("избꙑти"), imperfect_first_singular),
            Err(InflectionError::UnattestedUnreconstructableCell { .. })
        ));
    }

    #[test]
    fn suppletion_defectivity_and_sparse_evidence_remain_typed() {
        let iti = UniqueVerbIdentity::Iti.lexeme();
        assert_eq!(
            l_participle(
                &iti,
                LParticipleCell {
                    gender: crate::Gender::Masculine,
                    number: Number::Singular,
                }
            )
            .expect("suppletive l-participle")
            .text,
            "шьлъ"
        );

        let esmi = UniqueVerbIdentity::Esmi.lexeme();
        assert!(matches!(
            infinitive(&esmi),
            Err(InflectionError::HistoricallyInvalidCell { .. })
        ));

        let dovleti = UniqueVerbIdentity::Dovleti.lexeme();
        assert!(matches!(
            imperative(
                &dovleti,
                ImperativeCell {
                    person: Person::Second,
                    number: Number::Singular,
                },
            ),
            Err(InflectionError::UnattestedUnreconstructableCell { .. })
        ));

        let pleti = UniqueVerbIdentity::Pleti.lexeme();
        assert!(matches!(
            finite(
                &pleti,
                FiniteVerbCell {
                    tense: FiniteTense::Imperfect,
                    person: Person::First,
                    number: Number::Singular,
                }
            ),
            Err(InflectionError::UnattestedUnreconstructableCell { .. })
        ));
        assert_eq!(
            finite(
                &pleti,
                FiniteVerbCell {
                    tense: FiniteTense::Present,
                    person: Person::First,
                    number: Number::Plural,
                }
            )
            .expect("attested plěti present")
            .text,
            "плѣвемъ"
        );
    }

    #[test]
    fn exceptional_principal_parts_are_independent_source_goldens() {
        let second_singular = ImperativeCell {
            person: Person::Second,
            number: Number::Singular,
        };
        assert_eq!(
            imperative(&UniqueVerbIdentity::Vedeti.lexeme(), second_singular)
                .expect("athematic imperative")
                .text,
            "вѣжь"
        );
        assert_eq!(
            imperative(&UniqueVerbIdentity::Vopiti.lexeme(), second_singular)
                .expect("vowel-stem imperative")
                .text,
            "въпи"
        );
        assert_eq!(
            l_participle(
                &UniqueVerbIdentity::Leshti.lexeme(),
                LParticipleCell {
                    gender: crate::Gender::Masculine,
                    number: Number::Singular,
                },
            )
            .expect("root l-participle")
            .text,
            "леглъ"
        );

        let nominative = ParticipleCell {
            kind: ParticipleKind::PresentActive,
            adjective: AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        };
        let genitive = ParticipleCell {
            adjective: AdjectiveCell {
                case: Case::Genitive,
                ..nominative.adjective
            },
            ..nominative
        };
        assert_eq!(
            participle(&UniqueVerbIdentity::Deti.lexeme(), nominative)
                .expect("mixed root participle")
                .text,
            "дежѧ"
        );
        assert!(
            participle(&UniqueVerbIdentity::Deti.lexeme(), genitive)
                .expect("mixed root oblique")
                .text
                .starts_with("дежѫшт")
        );
        assert_eq!(
            participle(&UniqueVerbIdentity::Vopiti.lexeme(), nominative)
                .expect("iotated vowel participle")
                .text,
            "въпиѩ"
        );
        assert!(
            participle(&UniqueVerbIdentity::Vopiti.lexeme(), genitive)
                .expect("iotated vowel oblique")
                .text
                .starts_with("въпиѭшт")
        );
    }
}
