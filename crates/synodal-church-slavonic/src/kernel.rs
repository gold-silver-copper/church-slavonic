use synodal_church_slavonic_core::{
    DeterminerLexeme, Error, FiniteTense, FormSet, GrammarCell, NounLexeme, NumeralLexeme,
    OrthographyProfile, PronounLexeme, Result, VerbLexeme, aorist, decline_adjective,
    decline_determiner, decline_noun, decline_numeral, decline_participle, decline_pronoun,
    decline_verbal_noun, future, imperative, imperfect, infinitive, l_participle, present,
};

use synodal_church_slavonic_core::AdjectiveLexeme;

/// Validated lexical metadata accepted by the one productive generation
/// kernel used after either dictionary resolution or explicit specification.
pub(crate) enum ProductiveLexeme<'a> {
    Noun(&'a NounLexeme),
    Adjective(&'a AdjectiveLexeme),
    Determiner(&'a DeterminerLexeme),
    Numeral(&'a NumeralLexeme),
    Pronoun(&'a PronounLexeme),
    Verb(&'a VerbLexeme),
}

pub(crate) fn absent_synodal_supine() -> Error {
    Error::HistoricallyInvalidCell {
        reason: "the Russian/Synodal recension has no distinct supine: the historical category merged with the infinitive, including in motion-purpose constructions"
            .into(),
    }
}

pub(crate) fn generate_productive(
    lexeme: ProductiveLexeme<'_>,
    cell: GrammarCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    match (lexeme, cell) {
        (ProductiveLexeme::Noun(lexeme), GrammarCell::Noun(cell)) => {
            decline_noun(lexeme, cell, profile)
        }
        (ProductiveLexeme::Adjective(lexeme), GrammarCell::Adjective(cell)) => {
            decline_adjective(lexeme, cell, profile)
        }
        (ProductiveLexeme::Determiner(lexeme), GrammarCell::Determiner(cell)) => {
            decline_determiner(lexeme, cell, profile)
        }
        (ProductiveLexeme::Numeral(lexeme), GrammarCell::Numeral(cell)) => {
            decline_numeral(lexeme, cell, profile)
        }
        (ProductiveLexeme::Pronoun(lexeme), GrammarCell::Pronoun(cell)) => {
            decline_pronoun(lexeme, cell, profile)
        }
        (ProductiveLexeme::Verb(lexeme), GrammarCell::FiniteVerb(cell)) => match cell.tense {
            FiniteTense::Present => present(lexeme, cell.person, cell.number, profile),
            FiniteTense::Future => future(lexeme, cell.person, cell.number, profile),
            FiniteTense::Past => Err(Error::UnsupportedCell {
                reason: "the audited source-normalization past category has no productive target realization; request aorist or imperfect"
                    .into(),
            }),
            FiniteTense::Imperfect => imperfect(lexeme, cell.person, cell.number, profile),
            FiniteTense::Aorist => aorist(lexeme, cell.person, cell.number, profile),
        },
        (ProductiveLexeme::Verb(lexeme), GrammarCell::Imperative(cell)) => {
            imperative(lexeme, cell, profile)
        }
        (ProductiveLexeme::Verb(lexeme), GrammarCell::Infinitive) => infinitive(lexeme, profile),
        (ProductiveLexeme::Verb(lexeme), GrammarCell::LParticiple(cell)) => {
            l_participle(lexeme, cell, profile)
        }
        (ProductiveLexeme::Verb(lexeme), GrammarCell::Participle(cell)) => {
            decline_participle(lexeme, cell, profile)
        }
        (ProductiveLexeme::Verb(_), GrammarCell::Supine) => Err(absent_synodal_supine()),
        (ProductiveLexeme::Verb(lexeme), GrammarCell::VerbalNoun(cell)) => {
            decline_verbal_noun(lexeme, cell, profile)
        }
        (_, GrammarCell::LexicalForm) => Err(Error::UnsupportedCell {
            reason: "a lexical-form cell requires an exact form".into(),
        }),
        (_, GrammarCell::Indeclinable) => Err(Error::UnsupportedCell {
            reason: "an indeclinable cell requires an exact lexical form".into(),
        }),
        _ => Err(Error::UnsupportedCell {
            reason: "the requested cell is outside this lexical specification's grammatical inventory"
                .into(),
        }),
    }
}
