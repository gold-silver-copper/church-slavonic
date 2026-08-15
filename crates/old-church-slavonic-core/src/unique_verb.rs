//! Closed inventory of the nineteen unique Old Church Slavonic verb profiles.
//!
//! Polivanova 2023 §§417 and 516–605 treats these lexemes wholesale rather
//! than as members of the reusable irregular work-stem groups in §§434–440.
//! The inventory therefore lives above the productive verb rules: exact
//! present cells preserve each exceptional profile, independent principal
//! parts feed productive subsystems, and a source dash becomes an explicit
//! unreconstructable defect instead of missing metadata.

use crate::verb::VerbLexeme;
use crate::{
    AoristFormation, FiniteTense, FiniteVerbCell, ImperativeCell, ImperativeFormation,
    ImperfectFormation, ImperfectVariantPolicy, Number, ParticipleKind,
    PastActiveParticipleFormation, PastPassiveParticipleFormation, Person,
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

fn insert_imperative_singular(lexeme: &mut VerbLexeme, form: &str) {
    for person in [Person::Second, Person::Third] {
        lexeme.exact_forms.insert(
            VerbMorphologyCell::Imperative(ImperativeCell {
                person,
                number: Number::Singular,
            }),
            form.to_string(),
        );
    }
}

fn set_imperfect(lexeme: &mut VerbLexeme, stem: &str, formation: ImperfectFormation) {
    lexeme.stems.imperfect = Some(stem.to_string());
    lexeme.formations.imperfect = Some(formation);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
}

fn set_sigmatic_vowel_aorist(lexeme: &mut VerbLexeme, stem: &str, singular: &str) {
    lexeme.stems.aorist = Some(stem.to_string());
    lexeme.stems.aorist_second_third_singular = Some(singular.to_string());
    lexeme.formations.aorist = Some(AoristFormation::SigmaticVowel);
}

fn set_new_aorist(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.aorist = Some(stem.to_string());
    lexeme.formations.aorist = Some(AoristFormation::New);
}

fn set_imperative(lexeme: &mut VerbLexeme, stem: &str, formation: ImperativeFormation) {
    lexeme.stems.imperative = Some(stem.to_string());
    lexeme.formations.imperative = Some(formation);
}

fn set_l_participle(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.l_participle = Some(stem.to_string());
}

fn set_present_active(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PresentActiveParticipleFormation,
) {
    lexeme.stems.present_active_participle = Some(stem.to_string());
    lexeme.formations.present_active_participle = Some(formation);
}

fn set_present_passive(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PresentPassiveParticipleFormation,
) {
    lexeme.stems.present_passive_participle = Some(stem.to_string());
    lexeme.formations.present_passive_participle = Some(formation);
}

fn set_past_active(lexeme: &mut VerbLexeme, stem: &str, formation: PastActiveParticipleFormation) {
    lexeme.stems.past_active_participle = Some(stem.to_string());
    lexeme.formations.past_active_participle = Some(formation);
}

fn set_past_passive(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PastPassiveParticipleFormation,
) {
    lexeme.stems.past_passive_participle = Some(stem.to_string());
    lexeme.formations.past_passive_participle = Some(formation);
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
        ParticipleCell,
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
