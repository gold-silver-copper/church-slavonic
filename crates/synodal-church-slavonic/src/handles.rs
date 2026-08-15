use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Aspect, Case, Error, FiniteTense, FiniteVerbCell,
    FormSet, Gender, GrammarCell, ImperativeCell, LParticipleCell, LexemeId, MetadataField, Number,
    NumeralCell, NumeralKind, ParticipleCell, ParticipleTense, ParticipleVoice, Person,
    PronounCell, Result, SynodalWord, VerbSystem,
};

use crate::{
    Inflector, LexemeSummary, Paradigm, PartOfSpeech,
    paradigm::{
        adjective_cells, finite_cells, noun_cells, numeral_cells, participle_cells, pronoun_cells,
        verb_cells,
    },
    registry,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Capabilities {
    pub exact_forms: bool,
    pub productive_noun: bool,
    pub productive_adjective: bool,
    pub productive_determiner: bool,
    pub productive_pronoun: bool,
    pub present: bool,
    pub future: bool,
    pub past: bool,
    pub imperfect: bool,
    pub aorist: bool,
    pub imperative: bool,
    pub infinitive: bool,
    pub l_participle: bool,
    pub participle: bool,
    pub supine: bool,
    pub verbal_noun: bool,
    pub reverse_analysis: bool,
}

impl Capabilities {
    /// Returns the supported coarse system families in stable public order.
    /// Participles and verbal nouns are deliberately family-level capabilities;
    /// individual [`VerbSystem`] subtypes remain inspectable through paradigms.
    pub fn supported_systems(&self) -> impl Iterator<Item = &'static str> {
        [
            (self.productive_noun, "noun"),
            (self.productive_adjective, "adjective"),
            (self.productive_determiner, "determiner"),
            (self.productive_pronoun, "pronoun"),
            (self.present, "present"),
            (self.future, "future"),
            (self.past, "past"),
            (self.imperfect, "imperfect"),
            (self.aorist, "aorist"),
            (self.imperative, "imperative"),
            (self.infinitive, "infinitive"),
            (self.l_participle, "l-participle"),
            (self.participle, "participle"),
            (self.supine, "supine"),
            (self.verbal_noun, "verbal-noun"),
        ]
        .into_iter()
        .filter_map(|(supported, name)| supported.then_some(name))
    }

    pub(crate) fn for_summary(summary: &LexemeSummary, inflector: Inflector) -> Self {
        let noun = summary.part_of_speech() == PartOfSpeech::Noun;
        let verb = summary.part_of_speech() == PartOfSpeech::Verb;
        let id = summary.id();
        let productive_adjective = match summary.part_of_speech() {
            PartOfSpeech::Adjective => registry::adjective_lexeme(id).is_ok(),
            PartOfSpeech::Numeral => registry::ordinal_lexeme(id).is_ok(),
            _ => false,
        };
        let verb_metadata = verb
            .then(|| registry::verb_lexeme(id))
            .transpose()
            .ok()
            .flatten();
        Self {
            exact_forms: registry::has_exact_forms(id),
            productive_noun: noun
                && registry::noun_lexeme(id).is_ok()
                && (!registry::noun_uses_inherited_class(id)
                    || inflector.generation_policy()
                        != synodal_church_slavonic_core::GenerationPolicy::Strict),
            productive_adjective,
            productive_determiner: summary.part_of_speech() == PartOfSpeech::Determiner
                && registry::determiner_lexeme(id).is_ok(),
            productive_pronoun: summary.part_of_speech() == PartOfSpeech::Pronoun
                && registry::pronoun_lexeme(id).is_ok(),
            present: verb
                && (registry::has_exact_system(id, "present:")
                    || verb_metadata.as_ref().is_some_and(|metadata| {
                        metadata.present_stem.is_some()
                            && metadata.present_first_singular.is_some()
                            && metadata.present_third_plural.is_some()
                    })),
            future: verb && registry::has_exact_system(id, "future:"),
            past: verb && registry::has_exact_system(id, "past:"),
            imperfect: verb
                && (registry::has_exact_system(id, "imperfect:")
                    || registry::has_principal_part(id, "imperfect-stem")),
            aorist: verb
                && (registry::has_exact_system(id, "aorist:")
                    || registry::has_principal_part(id, "aorist-stem")),
            imperative: verb
                && (registry::has_exact_system(id, "imperative:")
                    || registry::has_principal_part(id, "imperative-stem")),
            infinitive: verb
                && (registry::has_exact_system(id, "infinitive") || verb_metadata.is_some()),
            l_participle: verb
                && (registry::has_exact_system(id, "l-participle:")
                    || registry::has_principal_part(id, "l-participle-stem")),
            participle: verb
                && (registry::has_exact_system(id, "participle:")
                    || registry::has_principal_part_prefix(id, "present-active-participle-")
                    || registry::has_principal_part_prefix(id, "past-active-participle-")
                    || registry::has_principal_part_prefix(id, "present-passive-participle-")
                    || registry::has_principal_part_prefix(id, "past-passive-participle-")),
            supine: verb && registry::has_exact_system(id, "supine"),
            verbal_noun: verb && registry::has_exact_system(id, "verbal-noun:"),
            reverse_analysis: true,
        }
    }
}

pub(crate) fn capabilities_by_id(id: &LexemeId, inflector: Inflector) -> Result<Capabilities> {
    let summary = inflector.from_id(id)?;
    Ok(Capabilities::for_summary(&summary, inflector))
}

pub(crate) fn missing_metadata_by_id(id: &LexemeId) -> Result<Vec<MetadataField>> {
    let summary = Inflector::default().from_id(id)?;
    Ok(missing_metadata(&summary))
}

macro_rules! identity_accessors {
    () => {
        #[must_use]
        pub fn id(&self) -> &LexemeId {
            self.summary.id()
        }

        #[must_use]
        pub fn lemma(&self) -> &str {
            self.summary.lemma()
        }

        #[must_use]
        pub fn capabilities(&self) -> Capabilities {
            Capabilities::for_summary(&self.summary, self.inflector)
        }

        #[must_use]
        pub fn missing_metadata(&self) -> Vec<MetadataField> {
            missing_metadata(&self.summary)
        }
    };
}

macro_rules! resolved_handle {
    ($handle:ident, $part_of_speech:expr) => {
        impl $handle {
            pub fn resolve(lemma: &str) -> Result<Self> {
                Self::resolve_with(lemma, Inflector::default())
            }

            pub fn resolve_with(lemma: &str, inflector: Inflector) -> Result<Self> {
                let summary = inflector.resolve(lemma)?;
                require_pos(&summary, $part_of_speech)?;
                Ok(Self { summary, inflector })
            }

            pub fn from_id(id: &LexemeId) -> Result<Self> {
                Self::from_id_with(id, Inflector::default())
            }

            pub fn from_id_with(id: &LexemeId, inflector: Inflector) -> Result<Self> {
                let summary = inflector.from_id(id)?;
                require_pos(&summary, $part_of_speech)?;
                Ok(Self { summary, inflector })
            }

            identity_accessors!();
        }
    };
}

#[derive(Clone, Debug)]
pub struct Noun {
    summary: LexemeSummary,
    inflector: Inflector,
}

resolved_handle!(Noun, PartOfSpeech::Noun);

impl Noun {
    pub fn form(&self, case: Case, number: Number, animacy: Animacy) -> Result<FormSet> {
        self.inflector.form_by_id(
            self.id(),
            GrammarCell::Noun(synodal_church_slavonic_core::NounCell {
                case,
                number,
                animacy,
            }),
        )
    }

    #[must_use]
    pub fn paradigm(&self, animacy: Animacy) -> Paradigm {
        Paradigm::build(self.inflector, self.summary.clone(), noun_cells(animacy))
    }
}

#[derive(Clone, Debug)]
pub struct Adjective {
    summary: LexemeSummary,
    inflector: Inflector,
}

resolved_handle!(Adjective, PartOfSpeech::Adjective);

impl Adjective {
    pub fn form(&self, cell: AdjectiveCell) -> Result<FormSet> {
        self.inflector
            .form_by_id(self.id(), GrammarCell::Adjective(cell))
    }

    #[must_use]
    pub fn paradigm(&self, form: AdjectiveForm) -> Paradigm {
        let cells = adjective_cells(form)
            .into_iter()
            .map(GrammarCell::Adjective);
        Paradigm::build(self.inflector, self.summary.clone(), cells)
    }
}

#[derive(Clone, Debug)]
pub struct Verb {
    summary: LexemeSummary,
    inflector: Inflector,
}

resolved_handle!(Verb, PartOfSpeech::Verb);

impl Verb {
    pub fn aspect(&self) -> Result<Aspect> {
        Ok(registry::verb_lexeme(self.id())?.aspect)
    }

    pub fn present(&self, person: Person, number: Number) -> Result<FormSet> {
        self.finite(FiniteTense::Present, person, number)
    }

    pub fn future(&self, person: Person, number: Number) -> Result<FormSet> {
        self.finite(FiniteTense::Future, person, number)
    }

    /// Looks up a reviewed exact finite-past form whose evidence does not
    /// distinguish aorist from imperfect.
    pub fn past(&self, person: Person, number: Number) -> Result<FormSet> {
        self.finite(FiniteTense::Past, person, number)
    }

    pub fn imperfect(&self, person: Person, number: Number) -> Result<FormSet> {
        self.finite(FiniteTense::Imperfect, person, number)
    }

    pub fn aorist(&self, person: Person, number: Number) -> Result<FormSet> {
        self.finite(FiniteTense::Aorist, person, number)
    }

    pub fn imperative(&self, person: Person, number: Number) -> Result<FormSet> {
        self.inflector.form_by_id(
            self.id(),
            GrammarCell::Imperative(ImperativeCell { person, number }),
        )
    }

    pub fn infinitive(&self) -> Result<FormSet> {
        self.inflector
            .form_by_id(self.id(), GrammarCell::Infinitive)
    }

    pub fn l_participle(&self, gender: Gender, number: Number) -> Result<FormSet> {
        self.inflector.form_by_id(
            self.id(),
            GrammarCell::LParticiple(LParticipleCell { gender, number }),
        )
    }

    #[must_use]
    pub fn paradigm(&self, tense: FiniteTense) -> Paradigm {
        Paradigm::build(self.inflector, self.summary.clone(), finite_cells(tense))
    }

    /// Builds one complete represented verb system, retaining invalid,
    /// unsupported, defective, and metadata-incomplete cells as rows.
    #[must_use]
    pub fn system_paradigm(&self, system: VerbSystem) -> Paradigm {
        Paradigm::build(self.inflector, self.summary.clone(), verb_cells(system))
    }

    /// Builds every represented verb-system inventory in stable order.
    #[must_use]
    pub fn all_system_paradigms(&self) -> Vec<(VerbSystem, Paradigm)> {
        VerbSystem::ALL
            .into_iter()
            .map(|system| (system, self.system_paradigm(system)))
            .collect()
    }

    /// Reports principal parts absent from this lexeme's productive
    /// background. Reviewed exact cells can still override individual rows.
    pub fn missing_principal_parts(&self, system: VerbSystem) -> Result<Vec<MetadataField>> {
        Ok(registry::verb_lexeme(self.id())?.missing_principal_parts(system))
    }

    fn finite(&self, tense: FiniteTense, person: Person, number: Number) -> Result<FormSet> {
        self.inflector.form_by_id(
            self.id(),
            GrammarCell::FiniteVerb(FiniteVerbCell {
                tense,
                person,
                number,
            }),
        )
    }
}

macro_rules! exact_handle {
    ($name:ident, $pos:expr, $cell:ty, $variant:ident) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            summary: LexemeSummary,
            inflector: Inflector,
        }

        resolved_handle!($name, $pos);

        impl $name {
            pub fn form(&self, cell: $cell) -> Result<FormSet> {
                self.inflector
                    .form_by_id(self.id(), GrammarCell::$variant(cell))
            }
        }
    };
}

exact_handle!(Pronoun, PartOfSpeech::Pronoun, PronounCell, Pronoun);
exact_handle!(Numeral, PartOfSpeech::Numeral, NumeralCell, Numeral);
exact_handle!(
    Determiner,
    PartOfSpeech::Determiner,
    AdjectiveCell,
    Determiner
);

impl Pronoun {
    #[must_use]
    pub fn paradigm(&self) -> Paradigm {
        let profiles = registry::pronoun_profiles(self.id());
        Paradigm::build(
            self.inflector,
            self.summary.clone(),
            pronoun_cells(&profiles),
        )
    }
}

impl Numeral {
    #[must_use]
    pub fn paradigm(&self, kind: NumeralKind) -> Paradigm {
        Paradigm::build(self.inflector, self.summary.clone(), numeral_cells(kind))
    }
}

impl Determiner {
    #[must_use]
    pub fn paradigm(&self, form: AdjectiveForm) -> Paradigm {
        let cells = adjective_cells(form)
            .into_iter()
            .map(GrammarCell::Determiner);
        Paradigm::build(self.inflector, self.summary.clone(), cells)
    }
}

#[derive(Clone, Debug)]
pub struct Participle {
    verb: Verb,
}

impl Participle {
    pub fn resolve(verb_lemma: &str) -> Result<Self> {
        Self::resolve_with(verb_lemma, Inflector::default())
    }

    pub fn resolve_with(verb_lemma: &str, inflector: Inflector) -> Result<Self> {
        Ok(Self {
            verb: Verb::resolve_with(verb_lemma, inflector)?,
        })
    }

    pub fn from_id(id: &LexemeId) -> Result<Self> {
        Self::from_id_with(id, Inflector::default())
    }

    pub fn from_id_with(id: &LexemeId, inflector: Inflector) -> Result<Self> {
        Ok(Self {
            verb: Verb::from_id_with(id, inflector)?,
        })
    }

    #[must_use]
    pub fn id(&self) -> &LexemeId {
        self.verb.id()
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.verb.lemma()
    }

    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        self.verb.capabilities()
    }

    #[must_use]
    pub fn missing_metadata(&self) -> Vec<MetadataField> {
        self.verb.missing_metadata()
    }

    pub fn form(&self, cell: ParticipleCell) -> Result<FormSet> {
        self.verb
            .inflector
            .form_by_id(self.id(), GrammarCell::Participle(cell))
    }

    #[must_use]
    pub fn paradigm(
        &self,
        tense: ParticipleTense,
        voice: ParticipleVoice,
        form: AdjectiveForm,
    ) -> Paradigm {
        Paradigm::build(
            self.verb.inflector,
            self.verb.summary.clone(),
            participle_cells(tense, voice, form),
        )
    }
}

fn missing_metadata(summary: &LexemeSummary) -> Vec<MetadataField> {
    let mut missing = Vec::new();
    if !registry::has_accent_data(summary.id()) {
        missing.push(MetadataField::AccentClass);
    }
    if summary.part_of_speech() != PartOfSpeech::Verb {
        return missing;
    }
    let id = summary.id();
    match registry::verb_lexeme(id) {
        Ok(metadata) => {
            if metadata.present_stem.is_none() {
                missing.push(MetadataField::PresentStem);
            }
            if metadata.present_first_singular.is_none() {
                missing.push(MetadataField::PresentFirstSingular);
            }
            if metadata.present_third_plural.is_none() {
                missing.push(MetadataField::PresentThirdPlural);
            }
        }
        Err(_) => {
            missing.extend([
                MetadataField::PresentStem,
                MetadataField::PresentFirstSingular,
                MetadataField::PresentThirdPlural,
            ]);
        }
    }
    for (field, system) in [
        (MetadataField::ImperfectStem, "imperfect-stem"),
        (MetadataField::AoristStem, "aorist-stem"),
        (MetadataField::ImperativeStem, "imperative-stem"),
        (MetadataField::LParticipleStem, "l-participle-stem"),
        (MetadataField::SupineStem, "supine-stem"),
        (MetadataField::ParticipleStem, "participle-stem"),
        (MetadataField::VerbalNounStem, "verbal-noun-stem"),
    ] {
        let has_principal_part = if field == MetadataField::ParticipleStem {
            registry::has_principal_part_prefix(id, "present-active-participle-")
                || registry::has_principal_part_prefix(id, "past-active-participle-")
                || registry::has_principal_part_prefix(id, "present-passive-participle-")
                || registry::has_principal_part_prefix(id, "past-passive-participle-")
        } else {
            registry::has_principal_part(id, system)
        };
        if !has_principal_part {
            missing.push(field);
        }
    }
    missing
}

pub(crate) fn resolve_summary(lemma: &str) -> Result<LexemeSummary> {
    registry::resolve(&SynodalWord::parse(lemma)?)
}

fn require_pos(summary: &LexemeSummary, expected: PartOfSpeech) -> Result<()> {
    if summary.part_of_speech() == expected {
        Ok(())
    } else {
        Err(Error::ContradictoryMetadata {
            reason: format!(
                "lexeme {} is {:?}, not {expected:?}",
                summary.id(),
                summary.part_of_speech()
            ),
        })
    }
}
