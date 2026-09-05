//! The table schema's cell geometry — the ONE copy of the index formulas
//! shared by the extractor (`extractor_legacy::cells` re-exports them) and the
//! `church-slavonic` runtime. Cell order:
//!
//! - noun (22): `number * 7 + case`, numbers `Singular, Dual, Plural`,
//!   cases `Nominative, Genitive, Dative, Accusative, Instrumental,
//!   Locative, Vocative`; cell 21 is the ACCENT-PATTERN cell;
//! - adjective (127): `((degree * 3 + gender) * 3 + number) * 7 + case`,
//!   degrees `Positive, Comparative`; cell 126 the accent-pattern cell;
//! - verb (558): four 9-cell finite blocks `Present, Imperfect, Aorist,
//!   Imperative` at `block * 9 + number * 3 + person`, the participle
//!   citations 36/37, the declined participle block at 38.., the four
//!   participle-stem cells 542..546, the present-stem override 546, the
//!   class override 547, the accent-pattern cell 548, and the l-participle
//!   (resultative) block at 549: `549 + gender * 3 + number`, nominative
//!   only;
//! - pronoun (119): the personal matrix (90: first person `number * 6 +
//!   case`, second `18 + …`, third `36 + (gender * 3 + number) * 6 + case`,
//!   six cases), the reflexive (90..96, by case; the nominative is blank),
//!   the clitic cells (96..119: first person `96 + number * 2 + {dat: 0,
//!   acc: 1}`, second `102 + …`, the third person's accusatives `108 +
//!   gender * 3 + number`, the reflexive's dative 117 and accusative 118);
//! - non-personal pronoun (54): `(gender * 3 + number) * 6 + case`, six
//!   cases (the vocative answers with the nominative).
//!
//! An accent-pattern token is `s<N>` (every form stresses its N-th vowel,
//! 0-based) or `e` (every form stresses its last vowel); it re-accents
//! whatever the letter-level resolution produced. Mobile paradigms carry
//! no token and stay stored.
//!
//! A fact cell holds a derived token or stem, never an attested form.

use crate::grammar::*;

pub const NOUN_ARITY: usize = 22;
pub const ADJ_ARITY: usize = 127;
pub const VERB_ARITY: usize = 558;
pub const PRONOUN_ARITY: usize = 119;

/// The first reflexive cell of the pronoun row (v1.2 part 3).
pub const REFLEXIVE_BASE: usize = 90;
/// The first clitic cell of the pronoun row.
pub const CLITIC_BASE: usize = 96;
pub const NPRON_ARITY: usize = 54;

/// The accent-pattern fact cell of each accented row.
pub const NOUN_ACCENT_CELL: usize = 21;
pub const ADJ_ACCENT_CELL: usize = 126;
pub const VERB_ACCENT_CELL: usize = 548;

/// The stored form cells the noun resolution may READ as facts: the
/// accusative singular (3) and dual (10). A Synodal masculine's accusative
/// answers the genitive shape by rule; a row whose stored accusative is
/// nominative-shaped instead (an inanimate) teaches its HIGHER accusative
/// cells the same shape (see `resolution::noun_fact_fallback`). Sources
/// only derive upward — cell 10 from 3, cell 17 from 3 or 10 — so the
/// lowest stored accusative is always the anchor and is never subtracted
/// by its own derivation.
pub const NOUN_SHAPE_SOURCE_CELLS: [usize; 3] = [3, 10, 17];

/// The present-stem override cell of the verb row.
pub const PRESENT_STEM_CELL: usize = 546;
/// The conjugation-class override cell of the verb row.
pub const VERB_CLASS_CELL: usize = 547;

pub const CASES: [Case; 7] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Locative,
    Case::Vocative,
];
pub const NUMBERS: [Number; 3] = [Number::Singular, Number::Dual, Number::Plural];
pub const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
pub const PERSONS: [Person; 3] = [Person::First, Person::Second, Person::Third];
pub const DEGREES: [Degree; 2] = [Degree::Positive, Degree::Comparative];
/// The finite blocks of the verb row, in cell order.
pub const VERB_BLOCKS: [(Tense, Form); 4] = [
    (Tense::Present, Form::Finite),
    (Tense::Imperfect, Form::Finite),
    (Tense::Aorist, Form::Finite),
    (Tense::Present, Form::Imperative),
];

pub fn noun_cell(case: &Case, number: &Number) -> usize {
    *number as usize * 7 + *case as usize
}

pub fn adj_cell(case: &Case, number: &Number, gender: &Gender, degree: &Degree) -> Option<usize> {
    let degree = match degree {
        Degree::Positive => 0,
        Degree::Comparative => 1,
        Degree::Superlative => return None,
    };
    Some(((degree * 3 + *gender as usize) * 3 + *number as usize) * 7 + *case as usize)
}

pub fn verb_cell(person: &Person, number: &Number, tense: &Tense, form: &Form) -> Option<usize> {
    let block = match (tense, form) {
        (Tense::Present, Form::Finite) => 0,
        (Tense::Imperfect, Form::Finite) => 1,
        (Tense::Aorist, Form::Finite) => 2,
        (_, Form::Imperative) => 3,
        (Tense::Present, Form::Participle) => return Some(36),
        (_, Form::Participle) => return Some(37),
        (_, Form::Infinitive) => return None,
    };
    Some(block * 9 + *number as usize * 3 + *person as usize)
}

pub fn pronoun_cell(person: &Person, number: &Number, gender: &Gender, case: &Case) -> usize {
    let case = if *case == Case::Vocative {
        0
    } else {
        *case as usize
    };
    match person {
        Person::First => *number as usize * 6 + case,
        Person::Second => 18 + *number as usize * 6 + case,
        Person::Third => 36 + (*gender as usize * 3 + *number as usize) * 6 + case,
    }
}

/// The reflexive pronoun's cell (себѐ has no number or gender; the
/// vocative answers with the nominative, which is blank).
pub fn reflexive_cell(case: &Case) -> usize {
    let case = if *case == Case::Vocative { 0 } else { *case as usize };
    REFLEXIVE_BASE + case
}

/// The clitic (enclitic) cell of a person's form, or `None` where the
/// language has none: the first and second persons have a dative and an
/// accusative clitic per number (мѝ/мѧ̀, ны̀; the dual and plural datives
/// are blank by rule), the third person an accusative per gender and
/// number (и҆̀, ю҆̀, є҆̀, ѧ҆̀).
pub fn clitic_cell(person: &Person, number: &Number, gender: &Gender, case: &Case) -> Option<usize> {
    let slot = match case {
        Case::Dative => 0,
        Case::Accusative => 1,
        _ => return None,
    };
    Some(match person {
        Person::First => CLITIC_BASE + *number as usize * 2 + slot,
        Person::Second => CLITIC_BASE + 6 + *number as usize * 2 + slot,
        Person::Third => {
            if slot == 0 {
                return None;
            }
            CLITIC_BASE + 12 + *gender as usize * 3 + *number as usize
        }
    })
}

/// The reflexive's clitic cell (сѝ, сѧ̀), or `None` for any other case.
pub fn reflexive_clitic_cell(case: &Case) -> Option<usize> {
    match case {
        Case::Dative => Some(CLITIC_BASE + 21),
        Case::Accusative => Some(CLITIC_BASE + 22),
        _ => None,
    }
}

/// What a pronoun cell index means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PronounCell {
    Full { person: Person, number: Number, gender: Gender, case: Case },
    Reflexive { case: Case },
    Clitic { person: Person, number: Number, gender: Gender, case: Case },
    ReflexiveClitic { case: Case },
}

/// Decode a pronoun cell index (0..119).
pub fn pronoun_features(cell: usize) -> PronounCell {
    if cell < REFLEXIVE_BASE {
        let (person, rest) = if cell < 18 {
            (Person::First, cell)
        } else if cell < 36 {
            (Person::Second, cell - 18)
        } else {
            (Person::Third, cell - 36)
        };
        return PronounCell::Full {
            person,
            number: NUMBERS[(rest / 6) % 3],
            gender: GENDERS[(rest / 6) / 3],
            case: CASES[rest % 6],
        };
    }
    if cell < CLITIC_BASE {
        return PronounCell::Reflexive { case: CASES[cell - REFLEXIVE_BASE] };
    }
    let rest = cell - CLITIC_BASE;
    if rest < 12 {
        let person = if rest < 6 { Person::First } else { Person::Second };
        let r = rest % 6;
        return PronounCell::Clitic {
            person,
            number: NUMBERS[r / 2],
            gender: Gender::Masculine,
            case: if r % 2 == 0 { Case::Dative } else { Case::Accusative },
        };
    }
    if rest < 21 {
        let r = rest - 12;
        return PronounCell::Clitic {
            person: Person::Third,
            number: NUMBERS[r % 3],
            gender: GENDERS[r / 3],
            case: Case::Accusative,
        };
    }
    PronounCell::ReflexiveClitic {
        case: if rest == 21 { Case::Dative } else { Case::Accusative },
    }
}

/// A non-personal pronoun cell: `(gender * 3 + number) * 6 + case`, the
/// vocative answering with the nominative.
pub fn npron_cell(gender: &Gender, number: &Number, case: &Case) -> usize {
    let case = if *case == Case::Vocative {
        0
    } else {
        *case as usize
    };
    (*gender as usize * 3 + *number as usize) * 6 + case
}

/// Decode a non-personal pronoun cell index (0..54).
pub fn npron_features(cell: usize) -> (Gender, Number, Case) {
    (
        GENDERS[cell / 18],
        NUMBERS[(cell / 6) % 3],
        CASES[cell % 6],
    )
}

/// The declined-participle block: cells 38.. of the verb row. `tense` is
/// collapsed to present/past (`Imperfect` and `Aorist` are both the past
/// participle, as in [`verb_cell`]).
pub fn participle_cell(
    voice: &Voice,
    series: &Series,
    tense: &Tense,
    gender: &Gender,
    number: &Number,
    case: &Case,
) -> usize {
    let series = match (voice, series) {
        (Voice::Active, Series::Short) => 0,
        (Voice::Active, Series::Long) => 1,
        (Voice::Passive, Series::Short) => 2,
        (Voice::Passive, Series::Long) => 3,
    };
    let tense = match tense {
        Tense::Present => 0,
        Tense::Imperfect | Tense::Aorist => 1,
    };
    38 + (((series * 2 + tense) * 3 + *gender as usize) * 3 + *number as usize) * 7 + *case as usize
}

/// The l-participle (resultative) block: nominative-only, `549 + gender * 3
/// + number`.
pub fn l_participle_cell(gender: &Gender, number: &Number) -> usize {
    549 + *gender as usize * 3 + *number as usize
}

/// Decode an l-participle cell index (549..558).
pub fn l_participle_features(cell: usize) -> (Gender, Number) {
    let rest = cell - 549;
    (GENDERS[rest / 3], NUMBERS[rest % 3])
}

/// The four participle-stem cells: a derived stem, not an attested form.
pub fn participle_stem_cell(voice: &Voice, tense: &Tense) -> usize {
    542 + match (voice, tense) {
        (Voice::Active, Tense::Present) => 0,
        (Voice::Active, _) => 1,
        (Voice::Passive, Tense::Present) => 2,
        (Voice::Passive, _) => 3,
    }
}

/// Decode a declined-participle cell index (38..542) back into its
/// features.
pub fn participle_features(cell: usize) -> (Voice, Series, bool, Gender, Number, Case) {
    let rest = cell - 38;
    let case = CASES[rest % 7];
    let rest = rest / 7;
    let number = NUMBERS[rest % 3];
    let rest = rest / 3;
    let gender = GENDERS[rest % 3];
    let rest = rest / 3;
    let past = rest % 2 == 1;
    let (voice, series) = match rest / 2 {
        0 => (Voice::Active, Series::Short),
        1 => (Voice::Active, Series::Long),
        2 => (Voice::Passive, Series::Short),
        _ => (Voice::Passive, Series::Long),
    };
    (voice, series, past, gender, number, case)
}

/// Decode a noun form cell index (0..21).
pub fn noun_features(cell: usize) -> (Case, Number) {
    (CASES[cell % 7], NUMBERS[cell / 7])
}

/// Decode an adjective form cell index (0..126).
pub fn adj_features(cell: usize) -> (Case, Number, Gender, Degree) {
    let case = CASES[cell % 7];
    let rest = cell / 7;
    let number = NUMBERS[rest % 3];
    let gender = GENDERS[(rest / 3) % 3];
    let degree = DEGREES[rest / 9];
    (case, number, gender, degree)
}

/// Decode a finite/imperative/citation cell index (0..38).
pub fn finite_features(cell: usize) -> (Person, Number, Tense, Form) {
    if cell >= 36 {
        let tense = if cell == 36 {
            Tense::Present
        } else {
            Tense::Aorist
        };
        return (Person::Third, Number::Singular, tense, Form::Participle);
    }
    let (tense, form) = VERB_BLOCKS[cell / 9];
    (PERSONS[cell % 3], NUMBERS[(cell % 9) / 3], tense, form)
}
