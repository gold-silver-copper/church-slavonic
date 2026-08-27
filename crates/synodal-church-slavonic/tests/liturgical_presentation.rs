//! The language-wide liturgical presentation of a cell (resolver
//! `present_liturgical_cell`), one test per root cause the gold-gap engine
//! burn-down fixed. Every expectation is a print of the pinned Elizabeth
//! Bible or an Alypy table cell.

use synodal_church_slavonic::{
    Animacy, Case, GrammarCell, Inflector, LexemeId, NounCell, Number, OrthographyProfile,
};

fn liturgical() -> Inflector {
    Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build()
}

fn printed(id: &str, cell: &str) -> Vec<String> {
    let cell: GrammarCell = cell.parse().expect("cell key");
    liturgical()
        .form_by_id(&LexemeId::from(id), cell)
        .unwrap_or_else(|error| panic!("{id} {cell:?}: {error}"))
        .variants()
        .iter()
        .map(|variant| variant.printed.clone())
        .collect()
}

fn noun(case: Case, number: Number) -> GrammarCell {
    GrammarCell::Noun(NounCell {
        case,
        number,
        animacy: Animacy::Inanimate,
    })
}

/// Gold contract §3.2: a reviewed print recorded verse-initially (`Ка́мень`,
/// Psalm 117:22) is the cell surface `ка́мень`; the capital is presentation.
#[test]
fn verse_initial_capital_of_a_reviewed_print_is_not_the_cell_surface() {
    let forms = printed(
        "synodal:noun:v07-c27905de175a0cde",
        "noun:nominative:singular:inanimate",
    );
    assert_eq!(forms[0], "ка́мень");
    assert!(forms.iter().all(|form| !form.starts_with('К')));
    // The same rule reaches irregular overrides (`Рѣ́хъ`) and exact pronoun
    // rows (`Си́мъ`).
    assert!(
        printed("synodal:verb:wikt-06af096688df", "aorist:first:singular")
            .contains(&"рѣ́хъ".to_owned())
    );
}

/// Alypy §2: word-initial о prints as the broad on by default — the Bible
/// prints 659 ѻ-initial types and no narrow о҆-initial type.
#[test]
fn word_initial_on_is_broad_by_default() {
    let forms = liturgical()
        .form("одръ", noun(Case::Nominative, Number::Singular))
        .expect("одръ");
    assert_eq!(forms.primary_text(), "ѻ҆́дръ");
}

/// Alypy §36: the plural genitive ending is always wide (дарѡ́въ ×21); the
/// dative ending is wide only where the instrumental singular shares its
/// letters (мꙋжє́мъ against мꙋ́жемъ) — feminine i-stems with -їю
/// instrumentals keep the narrow print (лю́демъ ×368, ме́рзостемъ ×6).
#[test]
fn wide_plural_endings_follow_the_declension() {
    let dar = liturgical()
        .form("даръ", noun(Case::Dative, Number::Plural))
        .expect("даръ dative plural");
    assert_eq!(dar.primary_text(), "дарѡ́мъ");
    let merzost = liturgical()
        .form("мерзость", noun(Case::Dative, Number::Plural))
        .expect("мерзость dative plural");
    assert_eq!(merzost.primary_text(), "ме́рзостємъ".replace('є', "е"));
    let rab_instrumental = liturgical()
        .form("рабъ", noun(Case::Instrumental, Number::Singular))
        .expect("рабъ instrumental singular");
    assert_eq!(rab_instrumental.primary_text(), "рабо́мъ");
}

/// Alypy §36: a generated plural or dual print homographic with a singular
/// print of the same lexeme (accents included, across genders for agreeing
/// words) takes the letter antistich: ю҆́нѡши / ю҆́ноши (gen sg), зє́млю /
/// зе́млю (acc sg), небє́снаѧ (pl neut) / небе́снаѧ (fem sg).
#[test]
fn plural_and_dual_homographs_take_the_letter_antistich() {
    assert_eq!(
        liturgical()
            .form("юноша", noun(Case::Nominative, Number::Plural))
            .expect("юноша nominative plural")
            .primary_text(),
        "ю҆́нѡши"
    );
    assert_eq!(
        liturgical()
            .form("землѧ", noun(Case::Genitive, Number::Dual))
            .expect("землѧ genitive dual")
            .primary_text(),
        "зє́млю"
    );
    assert_eq!(
        printed(
            "synodal:adjective:nebesn",
            "adjective:nominative:plural:neuter:inanimate:long:positive"
        )[0],
        "небє́снаѧ"
    );
    // No homograph, no substitution: во́ды (nom pl) against воды̀ (gen sg).
    assert_eq!(
        liturgical()
            .form("вода", noun(Case::Nominative, Number::Plural))
            .expect("вода nominative plural")
            .primary_text(),
        "во́ды"
    );
}

/// Alypy §5: a word-final grave has its pre-enclitic acute as a second
/// print of the same cell (менѐ / мене́ ×9), and a monosyllabic grave-bearing
/// pronoun its unaccented enclitic print (мѧ̀ / мѧ ×312). The isolated print
/// stays primary.
#[test]
fn enclitic_environment_prints_are_variants_after_the_isolated_print() {
    let mene = printed(
        "synodal:pronoun:az",
        "pronoun:genitive:singular:any:first:any",
    );
    assert_eq!(mene[0], "менѐ");
    assert!(mene.contains(&"мене́".to_owned()));
    let mya = printed(
        "synodal:pronoun:az",
        "pronoun:accusative:singular:any:first:any",
    );
    assert!(mya.contains(&"мѧ̀".to_owned()));
    assert!(mya.contains(&"мѧ".to_owned()));
    let ty = printed(
        "synodal:pronoun:ty",
        "pronoun:nominative:singular:any:second:any",
    );
    assert_eq!(ty[0], "ты̀");
    assert!(ty.contains(&"ты".to_owned()));
}

/// Reviewed third-person dual and plural prints are keyed with gender `any`;
/// a gender-specific request reaches them (Alypy §47 ѧ҆̀, и҆́хъ).
#[test]
fn gender_specific_pronoun_requests_reach_gender_neutral_exact_rows() {
    assert!(
        printed(
            "synodal:pronoun:on",
            "pronoun:accusative:dual:masculine:third:any"
        )
        .contains(&"ѧ҆̀".to_owned())
    );
    assert!(
        printed(
            "synodal:pronoun:on",
            "pronoun:genitive:plural:masculine:third:any"
        )
        .contains(&"и҆́хъ".to_owned())
    );
}

/// Alypy §§33, 37: the mixed -ц- masculine class offers the zero-ending
/// genitive plural beside -євъ (ѻ҆тє́цъ ×148 / ѻ҆тцє́въ ×28, конє́цъ ×21), and
/// the antistich distinguishes it from the nominative singular.
#[test]
fn mixed_ts_genitive_plural_offers_the_zero_ending() {
    let forms = printed("synodal:noun:infant", "noun:genitive:plural:animate");
    assert!(forms.contains(&"младе́нцєвъ".to_owned()), "{forms:?}");
    assert!(forms.contains(&"младе́нєцъ".to_owned()), "{forms:?}");
}
