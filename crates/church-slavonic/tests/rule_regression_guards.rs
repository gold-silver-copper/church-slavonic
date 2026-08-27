//! Guards against "obvious" regular-rule fixes that silently regress common
//! attested words.
//!
//! `church-slavonic-core`'s rules deliberately pick ONE class per lemma ending
//! and let the tables hold what that guess gets wrong (`отьць` -> the rule's
//! vocative is not `отьче`; `дьнь` -> the rule declines it as a jo-stem, the
//! source as an n-stem; `глаголати` -> the rule gives the thematic `глаголаѭ`
//! where the source has `глаголѭ`). Every attested word is nonetheless correct
//! at runtime, because the extractor tables any cell whose attested form
//! differs from the prediction.
//!
//! The catch is the bare-key RESERVATION policy (`extractor::extract`): when a
//! source attests both an irregular paradigm and one the rule predicts, the
//! regular one reserves the bare key and the irregular is demoted to a `_<n>`
//! key (`сꙑнъ`: the u-stem the rule predicts holds the bare key; the o-stem
//! variant is `сꙑнъ_2`). "Fixing" a rule so that it predicts a different
//! attested variant flips which paradigm holds the bare key.
//!
//! These tests pin the currently-correct output of the vulnerable words so that
//! a future rule "fix" fails loudly here instead of silently shipping a
//! renumbered paradigm. If you are here because one of these failed after
//! editing a core rule: regenerate the tables (`cargo xtask refresh-data`),
//! then decide whether the renumbering is the trade-off you wanted.

use church_slavonic::*;

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::Synodal;

fn noun(word: &str, case: Case, number: Number, r: Recension) -> String {
    ChurchSlavonic::noun(word, &case, &number, &r)
}

#[test]
fn tabled_cells_override_the_class_guess() {
    assert_eq!(
        noun("отьць", Case::Vocative, Number::Singular, OCS),
        "отьче"
    );
    assert_eq!(noun("дьнь", Case::Genitive, Number::Singular, OCS), "дьни");
    assert_eq!(
        noun("дьнь", Case::Instrumental, Number::Singular, OCS),
        "дьньмь"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "глаголати",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &OCS
        ),
        "глаголѭ"
    );
    // The short adjective's attested vocative is the nominative, not the
    // rule's `-е`.
    assert_eq!(
        ChurchSlavonic::adj(
            "новъ",
            &Case::Vocative,
            &Number::Singular,
            &Gender::Masculine,
            &Degree::Positive,
            &OCS
        ),
        "новъ"
    );
}

#[test]
fn a_regular_attested_paradigm_keeps_the_bare_key_for_the_rule() {
    // сꙑнъ attests the u-stem the rule predicts AND an o-stem variant: the
    // bare key is the rule's, the variant lives at `_2`.
    assert_eq!(noun("сꙑнъ", Case::Dative, Number::Singular, OCS), "сꙑнови");
    assert_eq!(noun("сꙑнъ_2", Case::Dative, Number::Singular, OCS), "сꙑноу");
    // Both keys agree on the base lemma.
    assert_eq!(
        noun("сꙑнъ_2", Case::Nominative, Number::Singular, OCS),
        "сꙑнъ"
    );
}

#[test]
fn untabled_regular_words_still_follow_the_rules() {
    // Plain o-stems have no row at all; the fallback is the whole story.
    assert_eq!(
        noun("градъ", Case::Genitive, Number::Singular, OCS),
        "града"
    );
    assert_eq!(noun("градъ", Case::Dative, Number::Singular, OCS), "градоу");
    assert_eq!(
        noun("градъ", Case::Locative, Number::Plural, OCS),
        "градѣхъ"
    );
    assert_eq!(
        noun("врагъ", Case::Locative, Number::Singular, SYN),
        "вразѣ"
    );
    assert_eq!(noun("мѫжь", Case::Genitive, Number::Singular, OCS), "мѫжа");
    assert_eq!(noun("село", Case::Nominative, Number::Dual, OCS), "селѣ");
}
