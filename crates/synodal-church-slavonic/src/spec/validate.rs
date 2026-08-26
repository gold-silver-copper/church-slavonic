use synodal_church_slavonic_core::{
    ActiveParticipleShortFormation, Error, GrammarCell, ParticiplePrincipalPart, Result,
    VerbConjugation,
};

use super::*;

pub(crate) fn validate_pair(left: bool, right: bool, label: &str) -> Result<()> {
    if left != right {
        return Err(Error::ContradictoryMetadata {
            reason: format!("{label} must be supplied together"),
        });
    }
    Ok(())
}

pub(crate) fn validate_context_cells(
    context: &SpecContext,
    accepts: impl Fn(GrammarCell) -> bool,
) -> Result<()> {
    if context
        .defective_cells
        .iter()
        .any(|cell| !accepts(cell.cell))
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a defective cell belongs to a different part of speech".into(),
        });
    }
    Ok(())
}

pub(crate) fn validate_participle(
    part: Option<&ParticiplePrincipalPart>,
    active: bool,
    present: bool,
    conjugation: VerbConjugation,
) -> Result<()> {
    let Some(part) = part else {
        return Ok(());
    };
    if !active && part.short_formation.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "passive short participles must not use an active citation-edge formation"
                .into(),
        });
    }
    if active && part.short_stem.is_some() != part.short_formation.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "an active short-participle stem requires its typed citation-edge formation"
                .into(),
        });
    }
    if let Some(formation) = part.short_formation {
        let formation_is_present = matches!(
            formation,
            ActiveParticipleShortFormation::PresentFirstUnpalatalized
                | ActiveParticipleShortFormation::PresentFirstPalatalized
                | ActiveParticipleShortFormation::PresentSecond
                | ActiveParticipleShortFormation::PresentAfterSibilant
        );
        if formation_is_present != present {
            return Err(Error::ContradictoryMetadata {
                reason: "short-participle formation does not match participle tense".into(),
            });
        }
        let conjugation_matches = match formation {
            ActiveParticipleShortFormation::PresentFirstUnpalatalized => {
                conjugation == VerbConjugation::FirstUnpalatalized
            }
            ActiveParticipleShortFormation::PresentFirstPalatalized => {
                conjugation == VerbConjugation::FirstPalatalized
            }
            ActiveParticipleShortFormation::PresentSecond
            | ActiveParticipleShortFormation::PresentAfterSibilant => {
                conjugation == VerbConjugation::Second
            }
            ActiveParticipleShortFormation::PastConsonant
            | ActiveParticipleShortFormation::PastVowel
            | ActiveParticipleShortFormation::PastIotated => true,
        };
        if !conjugation_matches {
            return Err(Error::ContradictoryMetadata {
                reason: "present participle formation contradicts the supplied conjugation class"
                    .into(),
            });
        }
    }
    Ok(())
}
