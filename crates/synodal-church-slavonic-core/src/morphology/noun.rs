use crate::{
    Animacy, Case, Error, FormSet, Gender, Number, OrthographyProfile, Result, SynodalWord,
};

use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NounDeclension {
    FirstHardMasculine,
    /// Historical `u`-stem members of the first declension, retaining the
    /// ordered `-ꙋ`, `-ови`, `-ове`, `-ов-`, and `-ми` alternatives described
    /// in Alypy §§37–38 (for example `сынъ` and `домъ`).
    FirstHardMasculineUStem,
    /// Ethnonyms in `-инъ`, whose plural drops `-ин-` and has nominative and
    /// vocative `-е`, for example `галїлеанинъ : галїлеане` (Alypy §37).
    FirstHardMasculineInEthnonym,
    /// The lexeme-specific mixed historical profile of `ꙋдъ`, which keeps
    /// its ordinary first-declension forms and can additionally use an
    /// extended `ꙋдес-` stem after the fourth-declension neuter analogy.
    FirstHardMasculineUdEs,
    /// First declension with a final velar and the reviewed first/second
    /// palatalizations of Alypy §34.
    FirstHardVelarMasculine,
    /// First-declension masculine with a sibilant stem and mixed endings.
    FirstMixedMasculine,
    /// First-declension masculine with a `ц`-final oblique stem and an
    /// independently supplied citation form, including mobile-`е` nouns such
    /// as `младенецъ : младенц-` and the bounded metathesized profile
    /// `жрецъ : жерц-`. Synodal `ц` combines with `ы`/`ъ` but not `о` (Alypy
    /// §8.c); the remaining mixed endings follow §§33–37.
    FirstMixedTsMasculine,
    FirstHardNeuter,
    FirstSoftMasculine,
    /// Agent nouns in `-тель`, retaining the ordinary soft paradigm plus the
    /// `-е` / `-їе` nominative-vocative plural variants of Alypy §37.
    FirstSoftMasculineAgentTel,
    /// The mixed `господь` profile: hard singular obliques, historical
    /// i-stem dual/plural endings, and the lexical vocative `господи`.
    FirstSoftMasculineLord,
    /// Masculine `j`-stems with a surface citation in `-й`, for example
    /// `край : кра-` and `славїй : славї-`.
    FirstSoftMasculineJ,
    /// Masculine nouns in `-ей` with the distinct `їерей` pattern from Alypy
    /// §§34 and 37, including genitive singular `-а` and plural `-є`.
    FirstSoftMasculineEy,
    FirstSoftNeuter,
    /// Soft neuters in `-ище`, whose locative plural admits ordered `-ахъ`,
    /// `-ихъ`, and `-ехъ` variants (Alypy §37).
    FirstSoftNeuterIshche,
    /// Soft neuters in `-їе`, whose dual/plural spelling and endings differ
    /// from the ordinary `море` pattern (for example `знаменїе`).
    FirstSoftNeuterIe,
    SecondHard,
    /// Second declension with a final velar and the §39 palatalization before
    /// `ѣ` in singular dative/locative and dual citation cells.
    SecondHardVelar,
    SecondSoft,
    /// Soft nouns in `-ѧ` after a vowel that retain the ancient `-ѧ`
    /// nominative/accusative plural, for example `молнїѧ` and `ѕмїѧ`.
    SecondSoftPostvocalicAncientPlural,
    /// Masculine names in `-їа`, with the §40 instrumental singular `-емъ`.
    SecondSoftMasculineIa,
    /// Feminine names in `-іа`, with the feminine instrumental singular
    /// `-іею`, for example `маріа : марі-` (Alypy §§32, 39–40).
    SecondSoftFeminineIa,
    /// Second-declension stems ending in a sibilant, with the mixed endings
    /// printed in Alypy §§39–40 (for example `юноша`).
    SecondMixed,
    ThirdFeminine,
    ThirdMasculine,
    /// Fourth-declension neuter whose citation form in `-ѧ` has an oblique
    /// stem in `-ен-`, for example `имѧ : имен-`.
    FourthNeuterEn,
    /// Fourth-declension neuter whose citation form in `-о` has an oblique
    /// stem in `-ес-`, for example `небо : небес-`.
    FourthNeuterEs,
    /// Extended `-ес-` neuters in `-о` that also admit a complete ordinary
    /// first-declension background without `-ес-` (Alypy §44).
    FourthNeuterEsAlternatingFirst,
    /// The paired-body `ѻко` / `ꙋхо` contract from Alypy §44: singular and
    /// plural use the independently supplied `-ес-` stem, while every dual
    /// cell uses its corresponding short `-ч-` / `-ш-` stem.
    FourthNeuterEsPairedDual,
    /// Fourth-declension neuter with an independently supplied extended stem
    /// in `-ат-`, for example `ѻтроча : ѻтрочат-`.
    FourthNeuterAt,
    /// Fourth-declension feminine whose citation form in `-и` has an oblique
    /// stem in `-ер-`, for example `мати : матер-`.
    FourthFeminineEr,
    /// The lexeme-specific modern `дщерь` identity with the historical
    /// nominative/vocative citation `дщи` and oblique `дщер-` stem.
    FourthFeminineErDaughter,
    /// Fourth-declension feminine with an independently supplied oblique stem
    /// in `-ов-` or `-в-`, for example `свекры : свекров-`.
    FourthFeminineOv,
    /// Modern `-овь` members of the `свекры` family whose full `-ов-` and
    /// syncopated `-в-` stems are distributed by cell, for example
    /// `церковь : церков- / церкв-` and `любовь : любов- / любв-`.
    /// `stem` is the independently supplied short `-в-` stem; the full stem
    /// is recoverable without ambiguity from the validated citation form.
    FourthFeminineOvSyncopating,
    /// Fourth-declension masculine with an independently supplied stem in
    /// `-ен-`, for example `степень : степен-`.
    FourthMasculineEn,
    /// The lexeme-specific syncopating paradigm of `день : дн- / ден-`.
    FourthMasculineEnDay,
    /// The lexeme-specific `камень` contract: the ordinary masculine `-ен-`
    /// paradigm plus only the alternatives cited in Alypy §43. The separate
    /// collective `каменїе` is never emitted by this contract.
    FourthMasculineEnKamen,
    /// Borrowed nouns whose supplied lemma is invariant in every licensed
    /// case and number, including the Hebrew names described in Alypy §37.
    Indeclinable,
}

impl NounDeclension {
    pub const ALL: [Self; 38] = [
        Self::FirstHardMasculine,
        Self::FirstHardMasculineUStem,
        Self::FirstHardMasculineInEthnonym,
        Self::FirstHardMasculineUdEs,
        Self::FirstHardVelarMasculine,
        Self::FirstMixedMasculine,
        Self::FirstMixedTsMasculine,
        Self::FirstHardNeuter,
        Self::FirstSoftMasculine,
        Self::FirstSoftMasculineAgentTel,
        Self::FirstSoftMasculineLord,
        Self::FirstSoftMasculineJ,
        Self::FirstSoftMasculineEy,
        Self::FirstSoftNeuter,
        Self::FirstSoftNeuterIshche,
        Self::FirstSoftNeuterIe,
        Self::SecondHard,
        Self::SecondHardVelar,
        Self::SecondSoft,
        Self::SecondSoftPostvocalicAncientPlural,
        Self::SecondSoftMasculineIa,
        Self::SecondSoftFeminineIa,
        Self::SecondMixed,
        Self::ThirdFeminine,
        Self::ThirdMasculine,
        Self::FourthNeuterEn,
        Self::FourthNeuterEs,
        Self::FourthNeuterEsAlternatingFirst,
        Self::FourthNeuterEsPairedDual,
        Self::FourthNeuterAt,
        Self::FourthFeminineEr,
        Self::FourthFeminineErDaughter,
        Self::FourthFeminineOv,
        Self::FourthFeminineOvSyncopating,
        Self::FourthMasculineEn,
        Self::FourthMasculineEnDay,
        Self::FourthMasculineEnKamen,
        Self::Indeclinable,
    ];
}

/// Numbers in which a noun is lexically licensed. This is lexical metadata,
/// not a request filter: asking for an absent number returns a historical-cell
/// error and remains visible in a complete paradigm.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NounNumberInventory {
    #[default]
    All,
    SingularOnly,
    DualOnly,
    PluralOnly,
    SingularAndDual,
    SingularAndPlural,
    DualAndPlural,
}

impl NounNumberInventory {
    #[must_use]
    pub const fn contains(self, number: Number) -> bool {
        matches!(
            (self, number),
            (Self::All, _)
                | (Self::SingularOnly, Number::Singular)
                | (Self::DualOnly, Number::Dual)
                | (Self::PluralOnly, Number::Plural)
                | (Self::SingularAndDual, Number::Singular | Number::Dual)
                | (Self::SingularAndPlural, Number::Singular | Number::Plural)
                | (Self::DualAndPlural, Number::Dual | Number::Plural)
        )
    }
}

/// Animacy values in which a noun is lexically licensed. This is independent
/// of the accusative-form request: an animate noun cannot acquire an
/// inanimate reverse analysis merely because the two surfaces are syncretic.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NounAnimacyInventory {
    #[default]
    All,
    AnimateOnly,
    InanimateOnly,
}

impl NounAnimacyInventory {
    #[must_use]
    pub const fn contains(self, animacy: Animacy) -> bool {
        matches!(
            (self, animacy),
            (Self::All, _)
                | (Self::AnimateOnly, Animacy::Animate)
                | (Self::InanimateOnly, Animacy::Inanimate)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounLexeme {
    pub lemma: SynodalWord,
    /// Productive stem. For fourth-declension classes this is the independently
    /// supplied extended oblique stem, not a stem inferred from the citation.
    pub stem: SynodalWord,
    pub gender: Gender,
    pub declension: NounDeclension,
    pub number_inventory: NounNumberInventory,
    pub animacy_inventory: NounAnimacyInventory,
}

impl NounLexeme {
    #[must_use]
    pub const fn new(
        lemma: SynodalWord,
        stem: SynodalWord,
        gender: Gender,
        declension: NounDeclension,
    ) -> Self {
        Self {
            lemma,
            stem,
            gender,
            declension,
            number_inventory: NounNumberInventory::All,
            animacy_inventory: NounAnimacyInventory::All,
        }
    }

    #[must_use]
    pub const fn with_number_inventory(mut self, inventory: NounNumberInventory) -> Self {
        self.number_inventory = inventory;
        self
    }

    #[must_use]
    pub const fn with_animacy_inventory(mut self, inventory: NounAnimacyInventory) -> Self {
        self.animacy_inventory = inventory;
        self
    }
}

pub fn decline_noun(
    lexeme: &NounLexeme,
    cell: crate::NounCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_noun_lexeme(lexeme)?;
    if !lexeme.number_inventory.contains(cell.number) {
        return Err(Error::HistoricallyInvalidCell {
            reason: format!(
                "noun {:?} is not licensed in {:?}",
                lexeme.lemma.canonical(),
                cell.number
            ),
        });
    }
    if !lexeme.animacy_inventory.contains(cell.animacy) {
        return Err(Error::HistoricallyInvalidCell {
            reason: format!(
                "noun {:?} is not licensed with {:?} animacy",
                lexeme.lemma.canonical(),
                cell.animacy
            ),
        });
    }
    let mut expanded = noun_surfaces(lexeme, cell)?;
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_like = noun_surfaces(
            lexeme,
            crate::NounCell {
                animacy: Animacy::Inanimate,
                ..cell
            },
        )?;
        if cell.number == Number::Plural
            && !matches!(
                lexeme.declension,
                NounDeclension::FourthFeminineEr
                    | NounDeclension::FourthFeminineErDaughter
                    | NounDeclension::FourthFeminineOv
                    | NounDeclension::FourthFeminineOvSyncopating
            )
        {
            let mut ordered = nominative_like;
            ordered.extend(expanded);
            expanded = ordered;
        } else {
            for form in nominative_like {
                if !expanded.contains(&form) {
                    expanded.push(form);
                }
            }
        }
        expanded.dedup();
    }
    normative_variants(
        expanded,
        noun_rule(lexeme.declension),
        profile,
        "noun-declension",
        lexeme.lemma.canonical(),
    )
}

pub(crate) fn noun_surfaces(lexeme: &NounLexeme, cell: crate::NounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Nominative as Nom, Vocative as Voc};
    use Number::Singular as Sg;

    if lexeme.declension == NounDeclension::Indeclinable {
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine
        && cell.number == Sg
        && matches!(cell.case, Nom | Acc)
        && (cell.case == Nom || cell.animacy == Animacy::Inanimate)
    {
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine
        && cell.number == Number::Plural
        && cell.case == Case::Genitive
    {
        // Alypy §§33, 37: beside -євъ the class keeps the zero-ending
        // genitive plural on the citation stem with its mobile vowel
        // (ѻ҆тє́цъ ×148 against ѻ҆тцє́въ ×28, конє́цъ ×21, а҆́гнєцъ ×5 in the
        // pinned Bible); the print distinguishes it from the nominative
        // singular by the §36 antistich.
        return Ok(vec![
            join(lexeme.stem.canonical(), "євъ"),
            lexeme.lemma.canonical().to_owned(),
        ]);
    }
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine
        && cell.number == Sg
        && cell.case == Voc
    {
        let stem = lexeme.stem.canonical();
        let palatalized = stem
            .strip_suffix('ц')
            .map_or_else(|| stem.to_owned(), |prefix| join(prefix, "ч"));
        return Ok(vec![join(&palatalized, "е")]);
    }
    if lexeme.declension == NounDeclension::FirstHardMasculineUdEs {
        let short_stem = lexeme.lemma.canonical().strip_suffix('ъ').ok_or_else(|| {
            Error::ContradictoryMetadata {
                reason: "the ꙋдъ mixed profile requires a citation in -ъ".into(),
            }
        })?;
        let ordinary = NounLexeme::new(
            lexeme.lemma.clone(),
            SynodalWord::parse(short_stem)?,
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
        );
        let mut surfaces = noun_surfaces(&ordinary, cell)?;
        if !matches!((cell.number, cell.case), (Sg, Nom | Acc | Voc)) {
            let extended = NounLexeme::new(
                SynodalWord::parse(join(short_stem, "о"))?,
                lexeme.stem.clone(),
                Gender::Neuter,
                NounDeclension::FourthNeuterEs,
            );
            surfaces.extend(noun_surfaces(&extended, cell)?);
            surfaces.dedup();
        }
        return Ok(surfaces);
    }
    if lexeme.declension == NounDeclension::FourthMasculineEnDay
        && cell.number == Sg
        && cell.case == Acc
    {
        return Ok(vec![if cell.animacy == Animacy::Animate {
            "дне".to_owned()
        } else {
            lexeme.lemma.canonical().to_owned()
        }]);
    }

    let citation_form = matches!(
        (lexeme.declension, cell.number, cell.case),
        (
            NounDeclension::FourthNeuterEn
                | NounDeclension::FourthNeuterEs
                | NounDeclension::FourthNeuterEsAlternatingFirst
                | NounDeclension::FourthNeuterEsPairedDual
                | NounDeclension::FourthNeuterAt,
            Sg,
            Nom | Acc | Voc
        ) | (
            NounDeclension::FourthFeminineEr
                | NounDeclension::FourthFeminineErDaughter
                | NounDeclension::FourthFeminineOv
                | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Nom | Voc
        ) | (
            NounDeclension::FourthMasculineEn
                | NounDeclension::FourthMasculineEnDay
                | NounDeclension::FourthMasculineEnKamen,
            Sg,
            Nom | Voc
        )
    );
    if citation_form {
        if lexeme.declension == NounDeclension::FourthFeminineErDaughter {
            let short_stem = lexeme
                .stem
                .canonical()
                .strip_suffix("ер")
                .unwrap_or_default();
            return Ok(vec![join(short_stem, "и")]);
        }
        if lexeme.declension == NounDeclension::FourthFeminineOvSyncopating && cell.case == Voc {
            return Ok(vec![
                lexeme.lemma.canonical().to_owned(),
                join(lexeme.stem.canonical(), "е"),
            ]);
        }
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }

    let stem = noun_stem(lexeme, cell);
    let mut surfaces = noun_endings(lexeme, cell)?
        .into_iter()
        .map(|ending| join(&stem, ending))
        .collect::<Vec<_>>();
    if lexeme.declension == NounDeclension::FourthNeuterEsAlternatingFirst {
        let short_stem = lexeme.stem.canonical().strip_suffix("ес").ok_or_else(|| {
            Error::ContradictoryMetadata {
                reason: "alternating -ес- neuters require an extended stem in -ес-".into(),
            }
        })?;
        let ordinary = NounLexeme::new(
            lexeme.lemma.clone(),
            SynodalWord::parse(short_stem)?,
            Gender::Neuter,
            NounDeclension::FirstHardNeuter,
        );
        surfaces.extend(noun_surfaces(&ordinary, cell)?);
        surfaces.dedup();
    }
    if lexeme.declension == NounDeclension::FourthMasculineEnKamen {
        let lexical_stem = lexeme.stem.canonical();
        let alternative = match (cell.number, cell.case, cell.animacy) {
            (Sg, crate::Case::Genitive, _) => Some(join(lexical_stem, "ѧ")),
            (Sg, crate::Case::Dative, _) => Some(join(lexical_stem, "ю")),
            (Number::Plural, Nom | Voc | Acc, Animacy::Inanimate) => Some(join(lexical_stem, "їѧ")),
            (Number::Plural, crate::Case::Locative, _) => Some(join(lexical_stem, "їѧхъ")),
            _ => None,
        };
        if let Some(alternative) = alternative {
            surfaces.push(alternative);
        }
    }
    if lexeme.declension == NounDeclension::FourthNeuterEsPairedDual
        && cell.number == Number::Dual
        && matches!(cell.case, Nom | Acc | Voc)
        && lexeme
            .stem
            .canonical()
            .strip_suffix("ес")
            .is_some_and(|stem| stem.ends_with('ч'))
    {
        if let Some(short_stem) = lexeme.stem.canonical().strip_suffix("ес") {
            let mut alternative = short_stem.to_owned();
            alternative.pop();
            alternative.push('ц');
            surfaces.push(join(&alternative, "ѣ"));
        }
    }
    if lexeme.declension == NounDeclension::FourthNeuterEsPairedDual
        && cell.number == Sg
        && cell.case == crate::Case::Locative
    {
        if let Some(short_stem) = lexeme.lemma.canonical().strip_suffix('о') {
            surfaces.push(join(&second_palatalize_final_velar(short_stem), "ѣ"));
        }
    }
    if lexeme.declension == NounDeclension::FirstSoftNeuterIe
        && cell.number == Number::Plural
        && cell.case == crate::Case::Instrumental
    {
        if let Some(short_stem) = lexeme
            .stem
            .canonical()
            .strip_suffix('ї')
            .or_else(|| lexeme.stem.canonical().strip_suffix('і'))
        {
            surfaces.push(join(short_stem, "ьми"));
            surfaces.push(join(short_stem, "ми"));
        }
    }
    surfaces.dedup();
    Ok(surfaces)
}

pub(crate) fn noun_stem(lexeme: &NounLexeme, cell: crate::NounCell) -> String {
    use Case::{Accusative as Acc, Nominative as Nom, Vocative as Voc};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    let stem = lexeme.stem.canonical();
    match lexeme.declension {
        NounDeclension::FirstHardMasculineInEthnonym if cell.number == Pl => {
            stem.strip_suffix("ин").unwrap_or(stem).to_owned()
        }
        NounDeclension::FourthMasculineEnDay
            if matches!(
                (cell.number, cell.case),
                (Du, crate::Case::Dative | crate::Case::Instrumental)
                    | (Pl, crate::Case::Instrumental)
            ) =>
        {
            "ден".to_owned()
        }
        NounDeclension::FirstHardVelarMasculine => match (cell.number, cell.case) {
            (Sg, Voc) => palatalize_final_velar(stem),
            (Sg, crate::Case::Locative) | (Pl, Nom | Voc | crate::Case::Locative) => {
                second_palatalize_final_velar(stem)
            }
            _ => stem.to_owned(),
        },
        NounDeclension::SecondHardVelar => match (cell.number, cell.case) {
            (Sg, crate::Case::Dative | crate::Case::Locative) | (Du, Nom | Acc | Voc) => {
                second_palatalize_final_velar(stem)
            }
            _ => stem.to_owned(),
        },
        NounDeclension::SecondMixed
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FirstSoftMasculineEy
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, crate::Case::Genitive | Acc | crate::Case::Instrumental)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FirstSoftNeuterIe
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Acc | Voc)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthNeuterEn
        | NounDeclension::FourthNeuterEs
        | NounDeclension::FourthNeuterEsAlternatingFirst
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthNeuterAt
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Voc)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthFeminineOv
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Voc)
            ) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FourthNeuterEsPairedDual if cell.number == Du => {
            stem.strip_suffix("ес").unwrap_or(stem).to_owned()
        }
        NounDeclension::FourthFeminineOvSyncopating => {
            let use_full_stem = matches!(
                (cell.number, cell.case),
                (Sg, Acc | crate::Case::Instrumental)
                    | (Du, crate::Case::Genitive | crate::Case::Locative)
            );
            let selected = if use_full_stem {
                lexeme
                    .lemma
                    .canonical()
                    .strip_suffix('ь')
                    .unwrap_or(lexeme.lemma.canonical())
            } else {
                stem
            };
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Voc)
            ) {
                last_e_as_wide_e(selected)
            } else {
                selected.to_owned()
            }
        }
        NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen
            if matches!(
                (cell.number, cell.case),
                (Du, Nom | Acc | Voc) | (Pl, Nom | Voc)
            ) || matches!(
                (cell.number, cell.case, cell.animacy),
                (Pl, Acc, Animacy::Inanimate)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        _ => stem.to_owned(),
    }
}

/// Validates the closed class/gender contract and the independently supplied
/// stem shape required by productive alternation rules.
pub fn validate_noun_lexeme(lexeme: &NounLexeme) -> Result<()> {
    let valid = matches!(
        (lexeme.declension, lexeme.gender),
        (
            NounDeclension::FirstHardMasculine
                | NounDeclension::FirstHardMasculineUStem
                | NounDeclension::FirstHardMasculineInEthnonym
                | NounDeclension::FirstHardMasculineUdEs,
            Gender::Masculine,
        ) | (NounDeclension::FirstHardVelarMasculine, Gender::Masculine)
            | (
                NounDeclension::FirstMixedMasculine | NounDeclension::FirstMixedTsMasculine,
                Gender::Masculine,
            )
            | (NounDeclension::FirstHardNeuter, Gender::Neuter)
            | (
                NounDeclension::FirstSoftMasculine
                    | NounDeclension::FirstSoftMasculineAgentTel
                    | NounDeclension::FirstSoftMasculineLord
                    | NounDeclension::FirstSoftMasculineJ
                    | NounDeclension::FirstSoftMasculineEy,
                Gender::Masculine,
            )
            | (
                NounDeclension::FirstSoftNeuter
                    | NounDeclension::FirstSoftNeuterIshche
                    | NounDeclension::FirstSoftNeuterIe,
                Gender::Neuter,
            )
            | (
                NounDeclension::SecondHard
                    | NounDeclension::SecondHardVelar
                    | NounDeclension::SecondSoft
                    | NounDeclension::SecondSoftPostvocalicAncientPlural
                    | NounDeclension::SecondMixed,
                Gender::Feminine | Gender::Masculine
            )
            | (NounDeclension::SecondSoftMasculineIa, Gender::Masculine)
            | (NounDeclension::SecondSoftFeminineIa, Gender::Feminine)
            | (NounDeclension::ThirdFeminine, Gender::Feminine)
            | (NounDeclension::ThirdMasculine, Gender::Masculine)
            | (
                NounDeclension::FourthNeuterEn
                    | NounDeclension::FourthNeuterEs
                    | NounDeclension::FourthNeuterEsAlternatingFirst
                    | NounDeclension::FourthNeuterEsPairedDual
                    | NounDeclension::FourthNeuterAt,
                Gender::Neuter
            )
            | (
                NounDeclension::FourthFeminineEr
                    | NounDeclension::FourthFeminineErDaughter
                    | NounDeclension::FourthFeminineOv
                    | NounDeclension::FourthFeminineOvSyncopating,
                Gender::Feminine
            )
            | (
                NounDeclension::FourthMasculineEn
                    | NounDeclension::FourthMasculineEnDay
                    | NounDeclension::FourthMasculineEnKamen,
                Gender::Masculine
            )
            | (NounDeclension::Indeclinable, _)
    );
    if !valid {
        return Err(Error::ContradictoryMetadata {
            reason: "declension and lexical gender are incompatible".into(),
        });
    }
    let lemma = lexeme.lemma.canonical();
    let stem = lexeme.stem.canonical();
    let valid_shape = match lexeme.declension {
        NounDeclension::FirstHardMasculineUStem => lemma.ends_with('ъ'),
        NounDeclension::FirstHardMasculineInEthnonym => {
            lemma.strip_suffix('ъ').is_some_and(|base| base == stem) && stem.ends_with("ин")
        }
        NounDeclension::FirstHardMasculineUdEs => lemma == "ꙋдъ" && stem == "ꙋдес",
        NounDeclension::FirstHardVelarMasculine => {
            lemma.ends_with('ъ')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'г' | 'к' | 'х'))
        }
        NounDeclension::FirstMixedMasculine => {
            lemma.ends_with('ъ')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'ж' | 'ч' | 'ш' | 'щ' | 'ц'))
        }
        NounDeclension::FirstMixedTsMasculine => {
            let citation_stem = lemma.strip_suffix('ъ').unwrap_or_default();
            let direct = citation_stem == stem;
            let mobile_e = stem
                .strip_suffix('ц')
                .is_some_and(|prefix| citation_stem == format!("{prefix}ец"));
            let transposed_e = stem
                .strip_suffix("ерц")
                .is_some_and(|prefix| citation_stem == format!("{prefix}рец"));
            stem.ends_with('ц') && (direct || mobile_e || transposed_e)
        }
        NounDeclension::FirstSoftMasculineJ => {
            lemma.strip_suffix('й').is_some_and(|prefix| prefix == stem) && !lemma.ends_with("ей")
        }
        NounDeclension::FirstSoftMasculineEy => {
            lemma.strip_suffix('й').is_some_and(|prefix| prefix == stem) && stem.ends_with('е')
        }
        NounDeclension::FirstSoftNeuterIe => {
            lemma.strip_suffix('е').is_some_and(|prefix| prefix == stem)
                && (stem.ends_with('ї') || stem.ends_with('і'))
        }
        NounDeclension::FirstSoftMasculineAgentTel => {
            lemma.strip_suffix('ь').is_some_and(|prefix| prefix == stem) && stem.ends_with("тел")
        }
        NounDeclension::FirstSoftMasculineLord => lemma == "господь" && stem == "господ",
        NounDeclension::FirstSoftNeuterIshche => {
            lemma.strip_suffix('е').is_some_and(|prefix| prefix == stem) && stem.ends_with("ищ")
        }
        NounDeclension::SecondHardVelar => {
            lemma.ends_with('а')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'г' | 'к' | 'х'))
        }
        NounDeclension::SecondMixed => {
            lemma.ends_with('а')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'ж' | 'ч' | 'ш' | 'щ' | 'ц'))
        }
        NounDeclension::SecondSoftPostvocalicAncientPlural => {
            lemma.ends_with('ѧ')
                && lemma.strip_suffix('ѧ').is_some_and(|prefix| prefix == stem)
                && stem.chars().last().is_some_and(|character| {
                    matches!(
                        character,
                        'а' | 'е'
                            | 'є'
                            | 'и'
                            | 'і'
                            | 'ї'
                            | 'о'
                            | 'ѡ'
                            | 'ꙋ'
                            | 'ѹ'
                            | 'ы'
                            | 'ѣ'
                            | 'ѧ'
                            | 'ю'
                            | 'ѵ'
                    )
                })
        }
        NounDeclension::SecondSoftMasculineIa | NounDeclension::SecondSoftFeminineIa => {
            // Alypy §§32, 39–40: the postvocalic а-declension covers stems in
            // -і-/-ї- (марі́а, и҆саі́а) and, in the target recension, the same
            // endings after -е- (галїле́а, і҆ꙋде́а).
            lemma.strip_suffix('а').is_some_and(|prefix| prefix == stem)
                && (stem.ends_with('ї') || stem.ends_with('і') || stem.ends_with('е'))
        }
        NounDeclension::FourthNeuterEn => lemma.ends_with('ѧ') && stem.ends_with("ен"),
        NounDeclension::FourthNeuterEs | NounDeclension::FourthNeuterEsAlternatingFirst => {
            lemma.strip_suffix('о').is_some_and(|short| {
                stem.strip_suffix("ес")
                    .is_some_and(|extended_short| extended_short == short)
            })
        }
        NounDeclension::FourthNeuterEsPairedDual => {
            lemma.ends_with('о')
                && stem
                    .strip_suffix("ес")
                    .is_some_and(|short| short.ends_with('ч') || short.ends_with('ш'))
        }
        NounDeclension::FourthNeuterAt => {
            (lemma.ends_with('а') || lemma.ends_with('ѧ')) && stem.ends_with("ат")
        }
        NounDeclension::FourthFeminineEr => lemma.ends_with('и') && stem.ends_with("ер"),
        NounDeclension::FourthFeminineErDaughter => lemma == "дщерь" && stem == "дщер",
        NounDeclension::FourthFeminineOv => {
            (lemma.ends_with('ы') || lemma.ends_with('ь'))
                && (stem.ends_with("ов") || stem.ends_with('в'))
                && !matches!(lemma, "любовь" | "любы")
        }
        NounDeclension::FourthFeminineOvSyncopating => lemma
            .strip_suffix("овь")
            .is_some_and(|prefix| stem == format!("{prefix}в")),
        NounDeclension::FourthMasculineEn => {
            lemma.ends_with("ень") && stem.ends_with("ен") && lemma != "камень"
        }
        NounDeclension::FourthMasculineEnDay => lemma == "день" && stem == "дн",
        NounDeclension::FourthMasculineEnKamen => lemma == "камень" && stem == "камен",
        NounDeclension::Indeclinable => !lemma.is_empty() && lemma == stem,
        _ => true,
    };
    if !valid_shape {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "lemma {lemma:?} and stem {stem:?} do not satisfy {:?}",
                lexeme.declension
            ),
        });
    }
    Ok(())
}

pub(crate) fn noun_rule(declension: NounDeclension) -> &'static str {
    match declension {
        NounDeclension::FirstHardMasculine => "SYN-NOUN-I-HARD-M-ALYPY-34",
        NounDeclension::FirstHardMasculineUStem => "SYN-NOUN-I-U-STEM-M-ALYPY-37-38",
        NounDeclension::FirstHardMasculineInEthnonym => "SYN-NOUN-I-HARD-M-IN-ETHNONYM-ALYPY-37",
        NounDeclension::FirstHardMasculineUdEs => "SYN-NOUN-I-M-UD-ES-ALYPY-44",
        NounDeclension::FirstHardVelarMasculine => "SYN-NOUN-I-HARD-VELAR-M-ALYPY-34",
        NounDeclension::FirstMixedMasculine => "SYN-NOUN-I-MIXED-M-ALYPY-33-34",
        NounDeclension::FirstMixedTsMasculine => "SYN-NOUN-I-MIXED-TS-M-ALYPY-8-33-37",
        NounDeclension::FirstHardNeuter => "SYN-NOUN-I-HARD-N-ALYPY-34",
        NounDeclension::FirstSoftMasculine => "SYN-NOUN-I-SOFT-M-ALYPY-34",
        NounDeclension::FirstSoftMasculineAgentTel => "SYN-NOUN-I-SOFT-M-TEL-AGENT-ALYPY-37",
        NounDeclension::FirstSoftMasculineLord => "SYN-NOUN-I-SOFT-M-LORD-ALYPY-35-41",
        NounDeclension::FirstSoftMasculineJ => "SYN-NOUN-I-SOFT-J-M-ALYPY-34-37",
        NounDeclension::FirstSoftMasculineEy => "SYN-NOUN-I-SOFT-EY-M-ALYPY-34-37",
        NounDeclension::FirstSoftNeuter => "SYN-NOUN-I-SOFT-N-ALYPY-34",
        NounDeclension::FirstSoftNeuterIshche => "SYN-NOUN-I-SOFT-N-ISHCHE-ALYPY-37",
        NounDeclension::FirstSoftNeuterIe => "SYN-NOUN-I-SOFT-IE-N-ALYPY-34-37",
        NounDeclension::SecondHard => "SYN-NOUN-II-HARD-ALYPY-39",
        NounDeclension::SecondHardVelar => "SYN-NOUN-II-HARD-VELAR-ALYPY-39-40",
        NounDeclension::SecondSoft => "SYN-NOUN-II-SOFT-ALYPY-39",
        NounDeclension::SecondSoftPostvocalicAncientPlural => {
            "SYN-NOUN-II-SOFT-POSTVOCALIC-ANCIENT-PL-ALYPY-40"
        }
        NounDeclension::SecondSoftMasculineIa => "SYN-NOUN-II-SOFT-M-IA-ALYPY-39-40",
        NounDeclension::SecondSoftFeminineIa => "SYN-NOUN-II-SOFT-F-IA-ALYPY-32-39-40",
        NounDeclension::SecondMixed => "SYN-NOUN-II-MIXED-ALYPY-39-40",
        NounDeclension::ThirdFeminine => "SYN-NOUN-III-F-ALYPY-41",
        NounDeclension::ThirdMasculine => "SYN-NOUN-III-M-ALYPY-41",
        NounDeclension::FourthNeuterEn => "SYN-NOUN-IV-N-EN-ALYPY-42-43",
        NounDeclension::FourthNeuterEs => "SYN-NOUN-IV-N-ES-ALYPY-42-43",
        NounDeclension::FourthNeuterEsAlternatingFirst => "SYN-NOUN-IV-N-ES-ALT-FIRST-ALYPY-42-44",
        NounDeclension::FourthNeuterEsPairedDual => "SYN-NOUN-IV-N-ES-PAIRED-DUAL-ALYPY-44",
        NounDeclension::FourthNeuterAt => "SYN-NOUN-IV-N-AT-ALYPY-42-43",
        NounDeclension::FourthFeminineEr => "SYN-NOUN-IV-F-ER-ALYPY-42-43",
        NounDeclension::FourthFeminineErDaughter => "SYN-NOUN-IV-F-ER-DAUGHTER-ALYPY-42-44",
        NounDeclension::FourthFeminineOv => "SYN-NOUN-IV-F-OV-ALYPY-42-44",
        NounDeclension::FourthFeminineOvSyncopating => "SYN-NOUN-IV-F-OV-SYNCOPATING-ALYPY-42-44",
        NounDeclension::FourthMasculineEn => "SYN-NOUN-IV-M-EN-ALYPY-42-44",
        NounDeclension::FourthMasculineEnDay => "SYN-NOUN-IV-M-EN-DAY-ALYPY-43",
        NounDeclension::FourthMasculineEnKamen => "SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43",
        NounDeclension::Indeclinable => "SYN-NOUN-INDECLINABLE-ALYPY-37",
    }
}

/// Read one Synodal vocalic column of the merged kernel
/// (`church_slavonic_core::noun`); the kernel's totality test guarantees a
/// non-empty ordered variant set per cell.
fn kernel_column(
    class: church_slavonic_core::noun::VocalicNounClass,
    cell: crate::NounCell,
) -> Vec<&'static str> {
    church_slavonic_core::noun::vocalic_ending(
        class,
        cell.case,
        cell.number,
        cell.animacy,
        church_slavonic_core::Recension::SynodalRussian,
    )
    .to_vec()
}

/// Read one Synodal consonant-stem column of the merged kernel
/// (`church_slavonic_core::noun_consonant`); an empty column marks a
/// citation cell that must be emitted from the supplied lemma.
fn consonant_column(
    class: church_slavonic_core::noun_consonant::ConsonantNounClass,
    cell: crate::NounCell,
) -> Result<Vec<&'static str>> {
    let endings = church_slavonic_core::noun_consonant::consonant_ending(
        class,
        cell.case,
        cell.number,
        cell.animacy,
        church_slavonic_core::Recension::SynodalRussian,
    );
    if endings.is_empty() {
        Err(fourth_declension_citation_error())
    } else {
        Ok(endings.to_vec())
    }
}

pub(crate) fn noun_endings(
    lexeme: &NounLexeme,
    cell: crate::NounCell,
) -> Result<Vec<&'static str>> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    use church_slavonic_core::noun::VocalicNounClass;
    use church_slavonic_core::noun_consonant::ConsonantNounClass;
    let animate_acc = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine {
        return Ok(match (cell.number, cell.case) {
            (Sg, Nom) => vec!["ъ"],
            (Sg, Gen) => vec!["а"],
            (Sg, Dat) => vec!["ꙋ", "еви"],
            (Sg, Acc) => vec![animate_acc("ъ", "а")],
            (Sg, Ins) => vec!["емъ"],
            (Sg, Loc) => vec!["и", "ѣ"],
            (Sg, Voc) => vec!["е"],
            (Du, Nom | Acc | Voc) => vec!["а"],
            (Du, Gen | Loc) => vec!["ꙋ"],
            (Du, Dat | Ins) => vec!["ема"],
            (Pl, Nom | Voc) => vec!["ы"],
            (Pl, Gen) => vec!["євъ"],
            (Pl, Dat) => vec!["ємъ"],
            (Pl, Acc) => vec![animate_acc("ы", "євъ")],
            (Pl, Ins) => vec!["ы", "ьми", "ами"],
            (Pl, Loc) => vec!["ѣхъ"],
        });
    }
    if lexeme.declension == NounDeclension::Indeclinable {
        return Ok(vec![""]);
    }
    if lexeme.declension == NounDeclension::FirstHardMasculineInEthnonym && cell.number == Pl {
        // Merged kernel: the shared -инъ singulative plural on the
        // syncopated stem (divergence
        // noun:in-singulative-inanimate-accusative on the accusative arm).
        return Ok(
            church_slavonic_core::noun_consonant::in_singulative_plural_ending(
                cell.case,
                cell.animacy,
                church_slavonic_core::Recension::SynodalRussian,
            )
            .to_vec(),
        );
    }
    if lexeme.declension == NounDeclension::FirstSoftMasculineLord {
        return Ok(match (cell.number, cell.case) {
            (Sg, Nom) => vec!["ь"],
            (Sg, Gen) => vec!["а"],
            (Sg, Dat) => vec!["ꙋ", "еви"],
            (Sg, Acc) => vec![animate_acc("ь", "а")],
            (Sg, Ins) => vec!["омъ"],
            (Sg, Loc) => vec!["ѣ"],
            (Sg, Voc) => vec!["и"],
            (Du, Nom | Acc | Voc) => vec!["и"],
            (Du, Gen | Loc) => vec!["їю", "ю"],
            (Du, Dat | Ins) => vec!["ьма"],
            (Pl, Nom | Voc) => vec!["їе"],
            (Pl, Gen) => vec!["ій", "ей"],
            (Pl, Dat) => vec!["ємъ"],
            (Pl, Acc) => vec![animate_acc("и", "ій")],
            (Pl, Ins) => vec!["ьми"],
            (Pl, Loc) => vec!["ехъ"],
        });
    }
    if lexeme.declension == NounDeclension::FirstSoftNeuterIshche
        && matches!((cell.number, cell.case), (Pl, Loc))
    {
        return Ok(vec!["ахъ", "ихъ", "ехъ"]);
    }
    if lexeme.declension == NounDeclension::FourthMasculineEnDay
        && matches!((cell.number, cell.case), (Du, Gen | Loc))
    {
        return Ok(vec!["їю", "ю"]);
    }
    if lexeme.declension == NounDeclension::FirstSoftMasculineAgentTel
        && matches!((cell.number, cell.case), (Pl, Nom | Voc))
    {
        // Merged kernel: the agent direct-plural variant set (divergence
        // noun:agent-plural-reinventory).
        return Ok(
            church_slavonic_core::noun_consonant::agent_direct_plural_ending(
                cell.case,
                cell.animacy,
                church_slavonic_core::Recension::SynodalRussian,
            )
            .to_vec(),
        );
    }
    let base_declension = match lexeme.declension {
        NounDeclension::FirstHardMasculineInEthnonym | NounDeclension::FirstHardMasculineUdEs => {
            NounDeclension::FirstHardMasculine
        }
        NounDeclension::FirstMixedTsMasculine => NounDeclension::FirstMixedMasculine,
        NounDeclension::FirstSoftMasculineAgentTel => NounDeclension::FirstSoftMasculine,
        NounDeclension::FirstSoftNeuterIshche => NounDeclension::FirstSoftNeuter,
        NounDeclension::FourthNeuterEsAlternatingFirst => NounDeclension::FourthNeuterEs,
        NounDeclension::FourthMasculineEnDay => NounDeclension::FourthMasculineEn,
        declension => declension,
    };
    // Merged kernel columns for the shared classes; the Synodal-only
    // subclasses keep their family tables and ordered pushes below.
    let merged: Option<Vec<&'static str>> = match base_declension {
        NounDeclension::FirstHardMasculine => {
            Some(kernel_column(VocalicNounClass::OHardMasculine, cell))
        }
        NounDeclension::FirstHardMasculineUStem => {
            Some(kernel_column(VocalicNounClass::UStemMasculine, cell))
        }
        NounDeclension::FirstHardNeuter => Some(kernel_column(VocalicNounClass::OHardNeuter, cell)),
        NounDeclension::FirstSoftMasculine => {
            Some(kernel_column(VocalicNounClass::JoSoftMasculine, cell))
        }
        NounDeclension::FirstSoftNeuter | NounDeclension::FirstSoftNeuterIe => {
            let mut endings = kernel_column(VocalicNounClass::JoSoftNeuter, cell);
            if base_declension == NounDeclension::FirstSoftNeuterIe {
                // Family -їе overrides over the merged soft-neuter column.
                endings = match (cell.number, cell.case) {
                    (Pl, Gen) => vec!["й"],
                    (Pl, Dat) => vec!["ємъ"],
                    (Pl, Loc) => vec!["ихъ"],
                    (Pl, Ins) => vec!["и"],
                    _ => endings,
                };
            }
            Some(endings)
        }
        NounDeclension::SecondHard => Some(kernel_column(VocalicNounClass::AHard, cell)),
        NounDeclension::SecondSoft => Some(kernel_column(VocalicNounClass::JaSoft, cell)),
        NounDeclension::ThirdFeminine => Some(kernel_column(VocalicNounClass::IFeminine, cell)),
        NounDeclension::ThirdMasculine => Some(kernel_column(VocalicNounClass::IMasculine, cell)),
        NounDeclension::FourthNeuterEn => {
            Some(consonant_column(ConsonantNounClass::NNeuter, cell)?)
        }
        NounDeclension::FourthNeuterEs => {
            Some(consonant_column(ConsonantNounClass::SNeuter, cell)?)
        }
        NounDeclension::FourthNeuterEsPairedDual => {
            // Family paired-body dual overrides over the merged -ес- column.
            let mut endings = consonant_column(ConsonantNounClass::SNeuter, cell)?;
            endings = match (cell.number, cell.case) {
                (Du, Gen | Loc) => vec!["їю"],
                (Du, Dat | Ins) => vec!["има"],
                _ => endings,
            };
            Some(endings)
        }
        NounDeclension::FourthNeuterAt => {
            Some(consonant_column(ConsonantNounClass::NtNeuter, cell)?)
        }
        NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter => {
            Some(consonant_column(ConsonantNounClass::RFeminine, cell)?)
        }
        NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating => {
            Some(consonant_column(ConsonantNounClass::VFeminine, cell)?)
        }
        NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen => {
            Some(consonant_column(ConsonantNounClass::NMasculine, cell)?)
        }
        _ => None,
    };
    if let Some(mut endings) = merged {
        // Lexeme-specific family pushes over the merged columns.
        match (lexeme.declension, cell.number, cell.case) {
            (NounDeclension::FourthMasculineEnDay, Sg, Dat) => endings.push("еви"),
            (NounDeclension::FourthMasculineEnDay, Pl, Nom | Voc) => endings.push("іе"),
            (NounDeclension::FourthMasculineEnDay, Pl, Gen) => endings.push("ей"),
            (NounDeclension::FourthMasculineEnKamen, Du, Dat | Ins) => endings.push("ема"),
            _ => {}
        }
        return Ok(endings);
    }
    let ending = match base_declension {
        NounDeclension::FirstHardVelarMasculine => match (cell.number, cell.case) {
            (Sg, Nom) => "ъ",
            (Sg, Gen) => "а",
            (Sg, Dat) => "ꙋ",
            (Sg, Acc) => animate_acc("ъ", "а"),
            (Sg, Ins) => "омъ",
            (Sg, Loc) => "ѣ",
            (Sg, Voc) => "е",
            (Du, Nom | Acc | Voc) => "а",
            (Du, Gen | Loc) => "ꙋ",
            (Du, Dat | Ins) => "ома",
            (Pl, Nom | Voc) if lexeme.stem.canonical().ends_with('к') => "ы",
            (Pl, Nom | Voc) => "и",
            (Pl, Gen) => "овъ",
            (Pl, Dat) => "омъ",
            (Pl, Acc) => animate_acc("и", "овъ"),
            (Pl, Ins) => "и",
            (Pl, Loc) => "ѣхъ",
        },
        NounDeclension::FirstMixedMasculine => match (cell.number, cell.case) {
            (Sg, Nom) => "ъ",
            (Sg, Gen) => "а",
            (Sg, Dat) => "ꙋ",
            (Sg, Acc) => animate_acc("ъ", "а"),
            (Sg, Ins) => "емъ",
            (Sg, Loc) => "и",
            (Sg, Voc) => "ꙋ",
            (Du, Nom | Acc | Voc) => "а",
            (Du, Gen | Loc) => "ꙋ",
            (Du, Dat | Ins) => "ема",
            (Pl, Nom | Voc) => "и",
            (Pl, Gen) => "ей",
            (Pl, Dat) => "емъ",
            (Pl, Acc) => animate_acc("ы", "ей"),
            (Pl, Ins) => "ы",
            (Pl, Loc) => "ахъ",
        },
        NounDeclension::FirstSoftMasculineJ => match (cell.number, cell.case) {
            (Sg, Nom) => "й",
            (Sg, Gen) => "ѧ",
            (Sg, Dat) => "ю",
            (Sg, Acc) => animate_acc("й", "ѧ"),
            (Sg, Ins) => "емъ",
            (Sg, Loc) => "и",
            (Sg, Voc) if lexeme.stem.canonical().ends_with('ї') => "е",
            (Sg, Voc) => "ю",
            (Du, Nom | Acc | Voc) => "ѧ",
            (Du, Gen | Loc) => "ю",
            (Du, Dat | Ins) => "ема",
            (Pl, Nom | Voc) => "и",
            (Pl, Gen) => "євъ",
            (Pl, Dat) => "ємъ",
            (Pl, Acc) => animate_acc("и", "євъ"),
            (Pl, Ins) => "и",
            (Pl, Loc) => "ехъ",
        },
        NounDeclension::FirstSoftMasculineEy => match (cell.number, cell.case) {
            (Sg, Nom) => "й",
            (Sg, Gen) => "а",
            (Sg, Dat) => "ю",
            (Sg, Acc) => animate_acc("й", "а"),
            (Sg, Ins) => "емъ",
            (Sg, Loc) => "и",
            (Sg, Voc) => "ю",
            (Du, Nom | Acc | Voc) => "а",
            (Du, Gen | Loc) => "ю",
            (Du, Dat | Ins) => "ема",
            (Pl, Nom | Voc) => "є",
            (Pl, Gen) => "й",
            (Pl, Dat) => "ємъ",
            (Pl, Acc) => animate_acc("и", "й"),
            (Pl, Ins) => "и",
            (Pl, Loc) => "ехъ",
        },
        NounDeclension::SecondHardVelar => match (cell.number, cell.case) {
            (Sg, Nom) => "а",
            (Sg, Gen) => "и",
            (Sg, Dat | Loc) => "ѣ",
            (Sg, Acc) => "ꙋ",
            (Sg, Ins) => "ою",
            (Sg, Voc) => "о",
            (Du, Nom | Acc | Voc) => "ѣ",
            (Du, Gen | Loc) => "ꙋ",
            (Du, Dat | Ins) => "ама",
            (Pl, Nom | Voc) => "и",
            (Pl, Gen) => "ъ",
            (Pl, Dat) => "амъ",
            (Pl, Acc) => animate_acc("и", "ъ"),
            (Pl, Ins) => "ами",
            (Pl, Loc) => "ахъ",
        },
        NounDeclension::SecondSoftPostvocalicAncientPlural => match (cell.number, cell.case) {
            (Sg, Nom) => "ѧ",
            (Sg, Gen | Dat | Loc) => "и",
            (Sg, Acc) => "ю",
            (Sg, Ins) => "ею",
            (Sg, Voc) => "е",
            (Du, Nom | Acc | Voc) => "и",
            (Du, Gen | Loc) => "ю",
            (Du, Dat | Ins) => "ѧма",
            (Pl, Nom | Voc) => "ѧ",
            (Pl, Gen) => "й",
            (Pl, Dat) => "ѧмъ",
            (Pl, Acc) => animate_acc("ѧ", "й"),
            (Pl, Ins) => "ѧми",
            (Pl, Loc) => "ѧхъ",
        },
        NounDeclension::SecondSoftMasculineIa | NounDeclension::SecondSoftFeminineIa => {
            match (cell.number, cell.case) {
                (Sg, Nom) => "а",
                (Sg, Gen | Dat | Loc) => "и",
                (Sg, Acc) => "ю",
                (Sg, Ins) if base_declension == NounDeclension::SecondSoftMasculineIa => "емъ",
                (Sg, Ins) => "ею",
                (Sg, Voc) => "е",
                (Du, Nom | Acc | Voc) => "и",
                (Du, Gen | Loc) => "ю",
                (Du, Dat | Ins) => "ѧма",
                (Pl, Nom | Voc) => "и",
                (Pl, Gen) => "й",
                (Pl, Dat) => "ѧмъ",
                (Pl, Acc) => animate_acc("и", "й"),
                (Pl, Ins) => "ѧми",
                (Pl, Loc) => "ѧхъ",
            }
        }
        NounDeclension::SecondMixed => match (cell.number, cell.case) {
            (Sg, Nom) => "а",
            (Sg, Gen | Dat | Loc) => "и",
            (Sg, Acc) => "ꙋ",
            (Sg, Ins) => "ею",
            (Sg, Voc) => "е",
            (Du, Nom | Acc | Voc) => "и",
            (Du, Gen | Loc) => "ꙋ",
            (Du, Dat | Ins) => "ама",
            (Pl, Nom | Voc) => "и",
            (Pl, Gen) => "ъ",
            (Pl, Dat) => "амъ",
            (Pl, Acc) => animate_acc("ы", "ъ"),
            (Pl, Ins) => "ами",
            (Pl, Loc) => "ахъ",
        },
        _ => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("unmapped noun declension base {base_declension:?}"),
            });
        }
    };
    let mut endings = vec![ending];
    match (lexeme.declension, cell.number, cell.case) {
        (NounDeclension::FirstHardVelarMasculine, Sg, Dat) => endings.push("ови"),
        (NounDeclension::FirstHardVelarMasculine, Pl, Gen) => endings.push("ъ"),
        (NounDeclension::FirstHardVelarMasculine, Pl, Ins) => endings.extend(["ми", "ами"]),
        (NounDeclension::FirstHardVelarMasculine, Pl, Loc) => endings.push("ахъ"),
        (NounDeclension::FirstMixedMasculine, Sg, Dat) => endings.push("еви"),
        (NounDeclension::FirstMixedMasculine, Sg, Loc) => endings.push("ѣ"),
        (NounDeclension::FirstMixedMasculine, Pl, Ins) => endings.extend(["ьми", "ами"]),
        (NounDeclension::FirstMixedMasculine, Pl, Nom | Voc) => endings.push("їе"),
        (NounDeclension::FirstSoftMasculineJ, Sg, Loc) => endings.push("ѣ"),
        (NounDeclension::FirstSoftMasculineEy, Sg, Dat) => endings.push("ови"),
        (NounDeclension::FirstSoftMasculineEy, Sg, Loc) => endings.push("ѣ"),
        (NounDeclension::FirstSoftMasculineEy, Sg, Voc) => endings.push("е"),
        (NounDeclension::FirstSoftMasculineEy, Du, Dat | Ins) => endings.push("ома"),
        (NounDeclension::FirstSoftMasculineEy, Pl, Dat) => endings.push("ѡмъ"),
        (NounDeclension::SecondMixed, Sg, Dat) => endings.push("ѣ"),
        _ => {}
    }
    Ok(endings)
}

pub(crate) fn fourth_declension_citation_error() -> Error {
    Error::UnsupportedCell {
        reason: "fourth-declension citation cells must be emitted from the supplied lemma".into(),
    }
}
