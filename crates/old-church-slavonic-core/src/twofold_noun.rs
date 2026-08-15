//! Closed lexical inventory for Polivanova's deformed twofold noun classes.
//!
//! Sections 329–332 and 345–348 define three productive subclasses over an
//! exhaustive 69 + 9 + 30 member dictionary inventory. The productive rules
//! live in [`crate::noun`]; this module preserves the lexical class assignment
//! required to apply them to a bare source lemma without shape guessing.

use crate::noun::NounLexeme;
use crate::{Animacy, Gender, NounClass, NumberRestriction};

const AGENT_MEMBERS: [&str; 69] = [
    "кесар҄ь",
    "винар҄ь",
    "властел҄ь",
    "вратар҄ь",
    "въздател҄ь",
    "възискател҄ь",
    "гонител҄ь",
    "въставител҄ь",
    "гръньчар҄ь",
    "доводител҄ь",
    "губител҄ь",
    "дател҄ь",
    "досадител҄ь",
    "дѣлател҄ь",
    "дѣлител҄ь",
    "жител҄ь",
    "жѧтел҄ь",
    "защитител҄ь",
    "зиждител҄ь",
    "зьдател҄ь",
    "избавител҄ь",
    "крьстител҄ь",
    "искусител҄ь",
    "исправител҄ь",
    "ицѣлител҄ь",
    "кл҄ѥветар҄ь",
    "кл҄ючар҄ь",
    "казател҄ь",
    "лател҄ь",
    "мѫчител҄ь",
    "мытар҄ь",
    "обрѣтател҄ь",
    "обадител҄ь",
    "обличител҄ь",
    "подадител҄ь",
    "подател҄ь",
    "побѣдител҄ь",
    "погребител҄ь",
    "подражател҄ь",
    "подъѩтел҄ь",
    "покровител҄ь",
    "правител҄ь",
    "приꙗтел҄ь",
    "проповѣдател҄ь",
    "просител҄ь",
    "прѣдател҄ь",
    "родител҄ь",
    "рыбар҄ь",
    "рьвьнител҄ь",
    "сѫдител҄ь",
    "свободител҄ь",
    "свѣтител҄ь",
    "свѧтител҄ь",
    "служител҄ь",
    "строител҄ь",
    "съврьшител҄ь",
    "съвѣдѣтел҄ь",
    "съдѣтел҄ь",
    "съзьдател҄ь",
    "съказател҄ь",
    "съпасител҄ь",
    "томител҄ь",
    "тьлител҄ь",
    "тѧжател҄ь",
    "учител҄ь",
    "хранител҄ь",
    "цѣлител҄ь",
    "цѣсар҄ь",
    "чистител҄ь",
];

const IN_MEMBERS: [&str; 9] = [
    "бол҄ꙗринъ",
    "гражданинъ",
    "жидовинъ",
    "жѧтел҄ꙗнинъ",
    "крьстиꙗнинъ",
    "исполинъ",
    "поганинъ",
    "сполинъ",
    "жител҄инъ",
];

const FEMININE_I_MEMBERS: [&str; 30] = [
    "балии",
    "благостын҄и",
    "благын҄и",
    "богын҄и",
    "господын҄и",
    "вѣтии",
    "гръдын҄и",
    "кръмьчии",
    "крьстиꙗнын҄и",
    "корабьчии",
    "крабии",
    "кън҄игъчии",
    "ладии",
    "льгын҄и",
    "милостын҄и",
    "мльнии",
    "поганын҄и",
    "правын҄и",
    "простын҄и",
    "пустын҄и",
    "рабын҄и",
    "сѫдии",
    "самъчии",
    "свинии",
    "свѧтын҄и",
    "сѫсѣдын҄и",
    "сокачии",
    "тысѫщи",
    "тысѧщи",
    "шаръчии",
];

/// One member of Polivanova's exhaustive deformed twofold noun inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TwofoldNounFamilyMember {
    class: NounClass,
    lemma: &'static str,
}

impl TwofoldNounFamilyMember {
    pub const COUNT: usize = 108;

    pub fn all() -> impl Iterator<Item = Self> {
        AGENT_MEMBERS
            .into_iter()
            .map(|lemma| Self {
                class: NounClass::TwofoldAgentMasculine,
                lemma,
            })
            .chain(IN_MEMBERS.into_iter().map(|lemma| Self {
                class: NounClass::TwofoldInMasculine,
                lemma,
            }))
            .chain(FEMININE_I_MEMBERS.into_iter().map(|lemma| Self {
                class: NounClass::TwofoldFeminineI,
                lemma,
            }))
    }

    pub fn classify_source_lemma(lemma: &str) -> Option<Self> {
        Self::all().find(|member| member.lemma == lemma)
    }

    pub const fn canonical_lemma(self) -> &'static str {
        self.lemma
    }

    pub const fn noun_class(self) -> NounClass {
        self.class
    }

    pub const fn source_class(self) -> &'static str {
        match self.class {
            NounClass::TwofoldAgentMasculine => "2/m*",
            NounClass::TwofoldInMasculine => "2/m**",
            NounClass::TwofoldFeminineI => "2/f*",
            _ => "unreachable",
        }
    }

    pub const fn source_section(self) -> &'static str {
        match self.class {
            NounClass::TwofoldAgentMasculine | NounClass::TwofoldInMasculine => "§§329–332",
            NounClass::TwofoldFeminineI => "§§345–348",
            _ => "unreachable",
        }
    }

    pub fn lexeme(self) -> NounLexeme {
        let gender = match self.class {
            NounClass::TwofoldAgentMasculine | NounClass::TwofoldInMasculine => Gender::Masculine,
            NounClass::TwofoldFeminineI => Gender::Feminine,
            _ => Gender::Feminine,
        };
        NounLexeme {
            lemma: self.lemma.to_string(),
            class: self.class,
            gender,
            // The printed profiles use nominative-like masculine accusatives.
            // A caller may supply an explicitly animate lexeme to the productive
            // rule, but the closed source inventory must reproduce its source.
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Case, NounCell, Number, RuleId, noun::decline};
    use std::collections::BTreeSet;

    #[test]
    fn source_inventory_is_exhaustive_unique_and_classed() {
        let members = TwofoldNounFamilyMember::all().collect::<Vec<_>>();
        assert_eq!(members.len(), TwofoldNounFamilyMember::COUNT);
        assert_eq!(
            members
                .iter()
                .map(|member| member.canonical_lemma())
                .collect::<BTreeSet<_>>()
                .len(),
            TwofoldNounFamilyMember::COUNT
        );
        assert_eq!(
            members
                .iter()
                .filter(|member| member.source_class() == "2/m*")
                .count(),
            69
        );
        assert_eq!(
            members
                .iter()
                .filter(|member| member.source_class() == "2/m**")
                .count(),
            9
        );
        assert_eq!(
            members
                .iter()
                .filter(|member| member.source_class() == "2/f*")
                .count(),
            30
        );
        for member in members {
            assert_eq!(
                TwofoldNounFamilyMember::classify_source_lemma(member.canonical_lemma()),
                Some(member)
            );
        }
        for excluded in [
            "господинъ",
            "чловѣчинъ",
            "воинъ",
            "съвоинъ",
            "окринъ",
            "чинъ",
        ] {
            assert_eq!(
                TwofoldNounFamilyMember::classify_source_lemma(excluded),
                None
            );
        }
    }

    #[test]
    fn every_member_reaches_every_cell_through_its_own_rule() {
        for member in TwofoldNounFamilyMember::all() {
            let expected_rule = match member.noun_class() {
                NounClass::TwofoldAgentMasculine => RuleId::NounTwofoldAgentMasculine,
                NounClass::TwofoldInMasculine => RuleId::NounTwofoldInMasculine,
                NounClass::TwofoldFeminineI => RuleId::NounTwofoldFeminineI,
                _ => panic!("non-twofold class in closed inventory"),
            };
            for number in Number::ALL {
                for case in Case::ALL {
                    let form = decline(&member.lexeme(), NounCell { case, number })
                        .unwrap_or_else(|error| panic!("{member:?} {case:?} {number:?}: {error}"));
                    assert_eq!(form.rule_id, expected_rule);
                    assert!(!form.text.is_empty());
                }
            }
        }
    }

    #[test]
    fn representative_profiles_preserve_the_source_deformations() {
        let form = |lemma, case, number| {
            let member = TwofoldNounFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing representative {lemma}"));
            decline(&member.lexeme(), NounCell { case, number })
                .unwrap_or_else(|error| panic!("{lemma}: {error}"))
                .text
        };
        assert_eq!(
            form("дѣлател҄ь", Case::Nominative, Number::Plural),
            "дѣлател҄ѥ"
        );
        assert_eq!(
            form("дѣлател҄ь", Case::Accusative, Number::Plural),
            "дѣлател҄ѩ"
        );
        assert_eq!(
            form("гражданинъ", Case::Nominative, Number::Plural),
            "граждане"
        );
        assert_eq!(
            form("гражданинъ", Case::Accusative, Number::Singular),
            "гражданинъ"
        );
        assert_eq!(
            form("гражданинъ", Case::Locative, Number::Plural),
            "гражданѣхъ"
        );
        assert_eq!(form("рабын҄и", Case::Nominative, Number::Singular), "рабын҄и");
        assert_eq!(form("рабын҄и", Case::Accusative, Number::Singular), "рабын҄ѭ");
    }
}
