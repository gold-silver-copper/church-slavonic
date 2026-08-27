//! Golden outputs for the REGULAR rule engine on representative un-tabled words.
//!
//! The `church-slavonic` crate's generated PHF tables hold ONLY irregular
//! exceptions; every regular word is served by these rules with no table row at
//! all. The `church-slavonic` crate's `rule_table_sync` test can only catch a
//! rule change that makes an existing bare TABLE row redundant — it is blind to
//! a rule change that breaks the fallback for the far larger set of words that
//! have no row precisely because the old rule predicted them. This golden test
//! pins those fallbacks so such a change fails `cargo test` deterministically,
//! without the sources. (It is NOT an exhaustive correctness spec — after any
//! rule edit, run `cargo xtask refresh-data` then `cargo xtask accuracy` to
//! re-measure.)
//!
//! Every row is one class in one recension, and every recension-conditioned
//! cell of the divergence registry that the rules kept appears at least once:
//! the `-ѥмь`/`-емъ` instrumental, the soft genitive/direct plurals, the
//! ja-stem genitive, the `-овъ` import, the neuter dual, the athematic
//! locative, the u-stem dissolution, the long-adjective contraction, the
//! short-oblique pronominalization, the soft vowel grades, the vocative
//! leveling, the dual `-вѣ`/`-ва` and third-dual leveling, the imperfect
//! contraction and hardening, the aorist `-шѧ`/`-ша`, the imperative vowel
//! grade, the participle citation contraction, and the copula cells.

use church_slavonic_core::ChurchSlavonicCore;
use church_slavonic_core::grammar::{Case, Degree, Form, Gender, Number, Person, Recension, Tense};
use church_slavonic_core::orthography;

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::Synodal;

#[test]
fn regular_noun_declensions() {
    use Case::*;
    use Number::*;
    for (word, r, case, number, expected) in [
        // hard o-stem masculine
        ("рабъ", OCS, Dative, Singular, "рабоу"),
        ("рабъ", SYN, Dative, Singular, "рабꙋ"),
        ("рабъ", OCS, Genitive, Plural, "рабъ"),
        ("рабъ", SYN, Genitive, Plural, "рабовъ"),
        ("рабъ", OCS, Accusative, Plural, "рабꙑ"),
        ("рабъ", SYN, Instrumental, Plural, "рабы"),
        ("врагъ", SYN, Locative, Singular, "вразѣ"),
        ("врагъ", OCS, Locative, Singular, "враѕѣ"),
        ("богъ", SYN, Vocative, Singular, "боже"),
        ("ѹченикъ", SYN, Nominative, Plural, "ѹченицы"),
        // hard o-stem neuter (noun:dual-direct-reshape)
        ("село", OCS, Nominative, Dual, "селѣ"),
        ("село", SYN, Nominative, Dual, "села"),
        ("село", SYN, Locative, Plural, "селѣхъ"),
        // soft jo-stems
        ("конь", OCS, Genitive, Singular, "конꙗ"),
        ("конь", SYN, Genitive, Singular, "конѧ"),
        ("конь", OCS, Instrumental, Singular, "конѥмь"),
        ("конь", SYN, Instrumental, Singular, "конемъ"),
        ("конь", OCS, Locative, Plural, "конихъ"),
        ("конь", SYN, Locative, Plural, "конехъ"),
        ("мѫжь", OCS, Genitive, Singular, "мѫжа"),
        ("мꙋжь", SYN, Dative, Singular, "мꙋжꙋ"),
        ("морѥ", OCS, Genitive, Singular, "морꙗ"),
        ("море", SYN, Genitive, Plural, "морей"),
        ("край", SYN, Genitive, Singular, "краѧ"),
        // a-stems and ja-stems
        ("жена", OCS, Accusative, Singular, "женѫ"),
        ("жена", SYN, Accusative, Singular, "женꙋ"),
        ("жена", OCS, Instrumental, Singular, "женоѭ"),
        ("жена", SYN, Instrumental, Singular, "женою"),
        ("рѫка", OCS, Dative, Singular, "рѫцѣ"),
        ("рꙋка", SYN, Genitive, Singular, "рꙋки"),
        ("землꙗ", OCS, Genitive, Singular, "землѩ"),
        ("землѧ", SYN, Genitive, Singular, "земли"),
        ("землꙗ", OCS, Nominative, Plural, "землѩ"),
        ("землѧ", SYN, Nominative, Plural, "земли"),
        ("доуша", OCS, Accusative, Singular, "доушѫ"),
        ("дꙋша", SYN, Accusative, Singular, "дꙋшꙋ"),
        // i-stems (noun:i-stem-instrumental-i-grade, noun:i-stem-vocative-leveling)
        ("кость", OCS, Instrumental, Singular, "костьѭ"),
        ("кость", SYN, Instrumental, Singular, "костїю"),
        ("кость", OCS, Vocative, Singular, "кости"),
        ("кость", SYN, Vocative, Singular, "косте"),
        ("кость", OCS, Genitive, Plural, "костии"),
        ("кость", SYN, Genitive, Plural, "костей"),
        // athematic
        ("имѧ", OCS, Instrumental, Singular, "именьмь"),
        ("имѧ", SYN, Instrumental, Singular, "именемъ"),
        ("имѧ", OCS, Nominative, Dual, "именѣ"),
        ("имѧ", SYN, Nominative, Dual, "имени"),
        ("мати", OCS, Genitive, Dual, "матероу"),
        ("мати", SYN, Genitive, Dual, "матерїю"),
        ("небо", OCS, Locative, Plural, "небесьхъ"),
        ("небо", SYN, Locative, Plural, "небесѣхъ"),
        ("свекрꙑ", OCS, Nominative, Singular, "свекрꙑ"),
        ("свекрꙑ", OCS, Dative, Singular, "свекръви"),
        // u-stem (OCS) and its Synodal dissolution
        ("сꙑнъ", OCS, Genitive, Singular, "сꙑноу"),
        ("сынъ", SYN, Genitive, Singular, "сына"),
        ("сꙑнъ", OCS, Instrumental, Singular, "сꙑнъмь"),
        ("сынъ", SYN, Instrumental, Singular, "сыномъ"),
    ] {
        assert_eq!(
            ChurchSlavonicCore::noun(word, &case, &number, &r),
            expected,
            "{word} {case:?} {number:?} {r:?}"
        );
    }
}

#[test]
fn regular_adjective_declensions_and_degrees() {
    use Case::*;
    use Gender::*;
    use Number::*;
    let pos = Degree::Positive;
    for (word, r, case, number, gender, degree, expected) in [
        ("новъ", OCS, Genitive, Singular, Masculine, pos, "нова"),
        ("новъ", SYN, Genitive, Singular, Masculine, pos, "нова"),
        (
            "новъ",
            OCS,
            Instrumental,
            Singular,
            Masculine,
            pos,
            "новомь",
        ),
        (
            "новъ",
            SYN,
            Instrumental,
            Singular,
            Masculine,
            pos,
            "новымъ",
        ),
        ("новъ", OCS, Dative, Plural, Feminine, pos, "новамъ"),
        ("новъ", SYN, Dative, Plural, Feminine, pos, "новымъ"),
        ("новъ", OCS, Accusative, Singular, Feminine, pos, "новѫ"),
        ("новъ", SYN, Accusative, Singular, Feminine, pos, "новꙋ"),
        ("синь", OCS, Dative, Singular, Neuter, pos, "синоу"),
        ("синь", SYN, Dative, Singular, Neuter, pos, "синю"),
        ("синь", OCS, Vocative, Singular, Masculine, pos, "сине"),
        ("синь", SYN, Vocative, Singular, Masculine, pos, "синь"),
        ("новꙑи", OCS, Nominative, Singular, Masculine, pos, "новꙑи"),
        ("новый", SYN, Nominative, Singular, Masculine, pos, "новый"),
        ("новꙑи", OCS, Dative, Singular, Masculine, pos, "новоуѥмоу"),
        ("новый", SYN, Dative, Singular, Masculine, pos, "новомꙋ"),
        ("новꙑи", OCS, Locative, Singular, Neuter, pos, "новѣѥмь"),
        ("новый", SYN, Locative, Singular, Neuter, pos, "новѣмъ"),
        ("новꙑи", OCS, Instrumental, Singular, Feminine, pos, "новѫѭ"),
        ("новый", SYN, Instrumental, Singular, Feminine, pos, "новою"),
        ("новꙑи", OCS, Genitive, Plural, Feminine, pos, "новꙑихъ"),
        ("новый", SYN, Genitive, Plural, Feminine, pos, "новыхъ"),
        ("синии", OCS, Genitive, Singular, Feminine, pos, "синѧѩ"),
        ("синїй", SYN, Genitive, Singular, Feminine, pos, "синїѧ"),
        ("синии", OCS, Dative, Singular, Feminine, pos, "синии"),
        ("синїй", SYN, Dative, Singular, Feminine, pos, "синей"),
        ("синии", OCS, Genitive, Dual, Masculine, pos, "синоую"),
        ("синїй", SYN, Genitive, Dual, Masculine, pos, "синюю"),
        ("благій", SYN, Nominative, Plural, Masculine, pos, "благїи"),
        (
            "новъ",
            OCS,
            Nominative,
            Singular,
            Masculine,
            Degree::Comparative,
            "новѣи",
        ),
        (
            "новъ",
            SYN,
            Nominative,
            Singular,
            Masculine,
            Degree::Comparative,
            "новѣй",
        ),
        (
            "новъ",
            SYN,
            Genitive,
            Singular,
            Masculine,
            Degree::Comparative,
            "новѣйша",
        ),
        (
            "новый",
            SYN,
            Genitive,
            Singular,
            Masculine,
            Degree::Comparative,
            "новѣйшагѡ",
        ),
        (
            "новъ",
            SYN,
            Nominative,
            Singular,
            Masculine,
            Degree::Superlative,
            "преновъ",
        ),
    ] {
        assert_eq!(
            ChurchSlavonicCore::adj(word, &case, &number, &gender, &degree, &r),
            expected,
            "{word} {case:?} {number:?} {gender:?} {degree:?} {r:?}"
        );
    }
}

#[test]
fn regular_verb_conjugations() {
    use Number::*;
    use Person::*;
    let fin = Form::Finite;
    for (word, r, person, number, tense, form, expected) in [
        (
            "нести",
            OCS,
            Second,
            Singular,
            Tense::Present,
            fin,
            "несеши",
        ),
        (
            "нести",
            SYN,
            Second,
            Singular,
            Tense::Present,
            fin,
            "несеши",
        ),
        ("нести", OCS, Third, Plural, Tense::Present, fin, "несѫтъ"),
        ("нести", SYN, Third, Plural, Tense::Present, fin, "несꙋтъ"),
        ("нести", OCS, First, Dual, Tense::Present, fin, "несевѣ"),
        ("нести", SYN, First, Dual, Tense::Present, fin, "несева"),
        ("нести", OCS, Third, Dual, Tense::Present, fin, "несете"),
        ("нести", SYN, Third, Dual, Tense::Present, fin, "несета"),
        ("знати", OCS, First, Singular, Tense::Present, fin, "знаѭ"),
        ("знати", SYN, First, Singular, Tense::Present, fin, "знаю"),
        ("знати", OCS, Third, Singular, Tense::Present, fin, "знаѥтъ"),
        ("знати", SYN, Third, Singular, Tense::Present, fin, "знаетъ"),
        (
            "хвалити",
            OCS,
            Second,
            Plural,
            Tense::Present,
            fin,
            "хвалите",
        ),
        (
            "хвалити",
            SYN,
            Third,
            Plural,
            Tense::Present,
            fin,
            "хвалѧтъ",
        ),
        (
            "кричати",
            SYN,
            First,
            Singular,
            Tense::Present,
            fin,
            "кричꙋ",
        ),
        (
            "кричати",
            SYN,
            Third,
            Plural,
            Tense::Present,
            fin,
            "кричатъ",
        ),
        (
            "видѣти",
            SYN,
            Second,
            Singular,
            Tense::Present,
            fin,
            "видиши",
        ),
        (
            "имѣти",
            SYN,
            Second,
            Singular,
            Tense::Present,
            fin,
            "имѣеши",
        ),
        (
            "цѣловати",
            OCS,
            Third,
            Singular,
            Tense::Present,
            fin,
            "цѣлоуѥтъ",
        ),
        ("рещи", SYN, Third, Singular, Tense::Present, fin, "речетъ"),
        // imperfect
        (
            "нести",
            OCS,
            Third,
            Singular,
            Tense::Imperfect,
            fin,
            "несѣаше",
        ),
        (
            "нести",
            SYN,
            Third,
            Singular,
            Tense::Imperfect,
            fin,
            "несѧше",
        ),
        (
            "нести",
            OCS,
            Second,
            Dual,
            Tense::Imperfect,
            fin,
            "несѣашета",
        ),
        ("нести", SYN, Second, Dual, Tense::Imperfect, fin, "несѧста"),
        (
            "нести",
            OCS,
            Third,
            Plural,
            Tense::Imperfect,
            fin,
            "несѣахѫ",
        ),
        ("нести", SYN, Third, Plural, Tense::Imperfect, fin, "несѧхꙋ"),
        (
            "знати",
            OCS,
            First,
            Plural,
            Tense::Imperfect,
            fin,
            "знаахомъ",
        ),
        (
            "знати",
            SYN,
            First,
            Plural,
            Tense::Imperfect,
            fin,
            "знахомъ",
        ),
        (
            "хвалити",
            OCS,
            First,
            Singular,
            Tense::Imperfect,
            fin,
            "хвалꙗахъ",
        ),
        (
            "хвалити",
            SYN,
            First,
            Singular,
            Tense::Imperfect,
            fin,
            "хвалѧхъ",
        ),
        (
            "видѣти",
            OCS,
            First,
            Singular,
            Tense::Imperfect,
            fin,
            "видѣахъ",
        ),
        (
            "видѣти",
            SYN,
            First,
            Singular,
            Tense::Imperfect,
            fin,
            "видѧхъ",
        ),
        (
            "рещи",
            SYN,
            Third,
            Singular,
            Tense::Imperfect,
            fin,
            "речаше",
        ),
        // aorist
        ("нести", OCS, First, Singular, Tense::Aorist, fin, "несохъ"),
        ("нести", SYN, Third, Singular, Tense::Aorist, fin, "несе"),
        ("нести", OCS, Third, Plural, Tense::Aorist, fin, "несошѧ"),
        ("нести", SYN, Third, Plural, Tense::Aorist, fin, "несоша"),
        ("знати", OCS, Third, Singular, Tense::Aorist, fin, "зна"),
        ("знати", SYN, Second, Plural, Tense::Aorist, fin, "знасте"),
        ("хвалити", OCS, First, Dual, Tense::Aorist, fin, "хвалиховѣ"),
        ("хвалити", SYN, First, Dual, Tense::Aorist, fin, "хвалихова"),
        ("хвалити", OCS, Third, Dual, Tense::Aorist, fin, "хвалисте"),
        ("хвалити", SYN, Third, Dual, Tense::Aorist, fin, "хвалиста"),
        ("рещи", OCS, Third, Singular, Tense::Aorist, fin, "рече"),
        // participles
        (
            "нести",
            OCS,
            Third,
            Singular,
            Tense::Present,
            Form::Participle,
            "несꙑ",
        ),
        (
            "нести",
            SYN,
            Third,
            Singular,
            Tense::Present,
            Form::Participle,
            "несый",
        ),
        (
            "знати",
            OCS,
            Third,
            Singular,
            Tense::Present,
            Form::Participle,
            "знаѩ",
        ),
        (
            "знати",
            SYN,
            Third,
            Singular,
            Tense::Present,
            Form::Participle,
            "знаѧ",
        ),
        (
            "хвалити",
            SYN,
            Third,
            Singular,
            Tense::Present,
            Form::Participle,
            "хвалѧ",
        ),
        (
            "нести",
            SYN,
            Third,
            Singular,
            Tense::Aorist,
            Form::Participle,
            "несъ",
        ),
        (
            "знати",
            OCS,
            Third,
            Singular,
            Tense::Aorist,
            Form::Participle,
            "знавъ",
        ),
        (
            "хвалити",
            OCS,
            Third,
            Singular,
            Tense::Aorist,
            Form::Participle,
            "хваль",
        ),
        (
            "хвалити",
            SYN,
            Third,
            Singular,
            Tense::Imperfect,
            Form::Participle,
            "хваливъ",
        ),
        // imperative
        (
            "нести",
            OCS,
            Second,
            Singular,
            Tense::Present,
            Form::Imperative,
            "неси",
        ),
        (
            "нести",
            OCS,
            First,
            Plural,
            Tense::Present,
            Form::Imperative,
            "несѣмъ",
        ),
        (
            "нести",
            SYN,
            First,
            Plural,
            Tense::Present,
            Form::Imperative,
            "несемъ",
        ),
        (
            "нести",
            OCS,
            Second,
            Dual,
            Tense::Present,
            Form::Imperative,
            "несѣта",
        ),
        (
            "нести",
            SYN,
            Second,
            Dual,
            Tense::Present,
            Form::Imperative,
            "несита",
        ),
        (
            "знати",
            OCS,
            Second,
            Plural,
            Tense::Present,
            Form::Imperative,
            "знаите",
        ),
        (
            "знати",
            SYN,
            Second,
            Plural,
            Tense::Present,
            Form::Imperative,
            "знайте",
        ),
        (
            "хвалити",
            SYN,
            Second,
            Singular,
            Tense::Present,
            Form::Imperative,
            "хвали",
        ),
        (
            "хвалити",
            SYN,
            Third,
            Singular,
            Tense::Present,
            Form::Imperative,
            "хвали",
        ),
        (
            "нести",
            SYN,
            Third,
            Singular,
            Tense::Present,
            Form::Infinitive,
            "нести",
        ),
        // the copula
        ("бꙑти", OCS, First, Singular, Tense::Present, fin, "ѥсмь"),
        ("быти", SYN, First, Singular, Tense::Present, fin, "єсмь"),
        ("бꙑти", OCS, Third, Plural, Tense::Present, fin, "сѫтъ"),
        ("быти", SYN, Third, Plural, Tense::Present, fin, "сꙋть"),
        ("бꙑти", OCS, First, Plural, Tense::Present, fin, "ѥсмъ"),
        ("быти", SYN, First, Plural, Tense::Present, fin, "єсмы"),
        ("бꙑти", OCS, First, Singular, Tense::Imperfect, fin, "бѣахъ"),
        ("быти", SYN, First, Singular, Tense::Imperfect, fin, "бѧхъ"),
        ("бꙑти", OCS, Third, Plural, Tense::Aorist, fin, "бѣшѧ"),
        ("быти", SYN, Third, Plural, Tense::Aorist, fin, "быша"),
        ("быти", SYN, Third, Singular, Tense::Aorist, fin, "бысть"),
        (
            "бꙑти",
            OCS,
            Third,
            Singular,
            Tense::Present,
            Form::Participle,
            "сꙑ",
        ),
        (
            "быти",
            SYN,
            Third,
            Singular,
            Tense::Aorist,
            Form::Participle,
            "бывъ",
        ),
        (
            "быти",
            SYN,
            Second,
            Plural,
            Tense::Present,
            Form::Imperative,
            "бꙋдите",
        ),
    ] {
        assert_eq!(
            ChurchSlavonicCore::verb(word, &person, &number, &tense, &form, &r),
            expected,
            "{word} {person:?} {number:?} {tense:?} {form:?} {r:?}"
        );
    }
}

#[test]
fn pronoun_matrix_and_realisation() {
    use Case::*;
    use Gender::*;
    use Number::*;
    use Person::*;
    for (person, number, gender, case, r, expected) in [
        (First, Singular, Masculine, Nominative, OCS, "азъ"),
        (First, Singular, Masculine, Instrumental, OCS, "мъноѭ"),
        (First, Singular, Masculine, Instrumental, SYN, "мною"),
        (First, Plural, Masculine, Accusative, OCS, "нꙑ"),
        (First, Plural, Masculine, Accusative, SYN, "насъ"),
        (Second, Dual, Masculine, Nominative, OCS, "ва"),
        (Second, Dual, Masculine, Nominative, SYN, "вы"),
        (Second, Singular, Masculine, Accusative, OCS, "тѧ"),
        (Second, Singular, Masculine, Accusative, SYN, "тебе"),
        (Third, Singular, Masculine, Genitive, OCS, "ѥго"),
        (Third, Singular, Masculine, Genitive, SYN, "єгѡ"),
        (Third, Singular, Feminine, Accusative, OCS, "ѭ"),
        (Third, Singular, Feminine, Accusative, SYN, "ю"),
        (Third, Singular, Masculine, Locative, OCS, "ѥмь"),
        (Third, Singular, Masculine, Locative, SYN, "немъ"),
        (Third, Plural, Feminine, Nominative, OCS, "онꙑ"),
        (Third, Plural, Feminine, Nominative, SYN, "онѣ"),
        (Third, Plural, Neuter, Accusative, OCS, "ꙗ"),
        (Third, Plural, Masculine, Accusative, SYN, "ихъ"),
    ] {
        assert_eq!(
            ChurchSlavonicCore::pronoun(&person, &number, &gender, &case, &r),
            expected,
            "{person:?} {number:?} {gender:?} {case:?} {r:?}"
        );
    }
    // realisation takes a rule answer across the recension boundary
    let ocs = ChurchSlavonicCore::noun("рѫка", &Dative, &Singular, &OCS);
    assert_eq!(orthography::realise(&ocs, &SYN), "рꙋцѣ");
    let syn = ChurchSlavonicCore::noun("рꙋка", &Dative, &Singular, &SYN);
    // the reverse fold is spelling-only: the big yus is not recoverable
    assert_eq!(orthography::realise(&syn, &OCS), "роуцѣ");
    assert_eq!(
        orthography::comparison_key(&ocs),
        orthography::comparison_key(&syn)
    );
}
