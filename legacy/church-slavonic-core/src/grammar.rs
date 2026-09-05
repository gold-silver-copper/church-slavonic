//! The grammatical feature enums that parameterize every inflection query.
//!
//! These are deliberately minimal — only distinctions Church Slavonic
//! morphology actually realizes are present (no Future tense: the future is
//! periphrastic or the perfective present, the caller's composition job; no
//! Animacy: the rules answer the nominative-shaped accusative and the tables
//! hold the genitive-shaped animate cells; no Voice or Mood beyond what
//! [`Form`] carries). All enums are `Copy` and passed by reference for API
//! stability with the `english` crate family.

/// Grammatical number. The dual is a live category in both recensions
/// (Synodal re-inventories its endings, it does not drop them).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Number {
    Singular,
    Dual,
    Plural,
}

/// The seven cases of the nominal paradigm. The vocative is a real cell for
/// nouns and adjectives; pronouns answer it with the nominative.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
    Vocative,
}

/// Agreement gender, for adjectives and the third-person pronoun.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Grammatical person, for verbs and the personal pronoun.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Morphological tense realized by simple finite forms: the present and the
/// two synthetic pasts. The perfect, pluperfect and future are periphrastic
/// (`l`-participle + copula, `бꙋдꙋ` + infinitive), so callers compose them.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tense {
    Present,
    /// The durative past (`несѧхъ`, OCS `несѣахъ`).
    Imperfect,
    /// The narrative past (`несохъ`, `дѣлахъ`).
    Aorist,
}

/// Verb form requested from the conjugator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Form {
    /// Finite form agreeing with person and number.
    Finite,
    /// The active participle's masculine nominative-singular citation form.
    /// Combined with [`Tense::Present`] for the present participle
    /// (`несый`), or with [`Tense::Aorist`] (or [`Tense::Imperfect`]) for
    /// the past participle (`несъ`, `дѣлавъ`).
    Participle,
    /// Bare infinitive lemma.
    Infinitive,
    /// The imperative, agreeing with person and number (`неси`, `несите`).
    Imperative,
}

/// Participle voice.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Voice {
    Active,
    Passive,
}

/// The participle's declension series: the short (nominal) declension
/// against the long (compound) one — the same split the adjectives make.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Series {
    Short,
    Long,
}

/// Degree for adjectives.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

/// The orthographic-morphological variety a query is answered in. Every rule
/// takes one: the two recensions share the paradigm skeleton but differ in
/// spelling (`ꙑ`/`ы`, `оу`/`ꙋ`, the nasals) and, at named cells, in the
/// ending itself (the Synodal `-емъ` instrumental against OCS `-ѥмь`, the
/// levelled dual, the contracted long adjective).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Recension {
    /// The canonical Old Church Slavonic of the grammars (Polivanova).
    OldChurchSlavonic,
    /// The Russian Synodal print (Alypy).
    Synodal,
}
