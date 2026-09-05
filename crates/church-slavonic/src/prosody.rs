//! The phonological word: a host with its enclitics and proclitics is one
//! accentual unit, and the print's oxia against varia is decided over the
//! unit (Землѧ́же: землѧ̀ + же). The lexicon says which words lean
//! ([`crate::grammar::Prosody`] from `stems=pros=`); this module groups a
//! token sequence into units for a renderer or a generator. Everything
//! word-level — the number mark, the kamora, the monosyllabic varia —
//! stays where the word has it; the unit only decides the final varia.

use crate::grammar::Prosody;

/// One accentual unit over a token sequence: the indices of the host, of
/// the proclitics before it and of the enclitics after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhonologicalWord {
    pub host: usize,
    pub proclitics: Vec<usize>,
    pub enclitics: Vec<usize>,
}

/// Group a sequence of words (each with its prosody) into phonological
/// words: a proclitic attaches to the next tonic word, an enclitic to the
/// previous unit; a clitic with nothing to lean on is a unit of its own.
pub fn words(prosodies: &[Prosody]) -> Vec<PhonologicalWord> {
    let mut out: Vec<PhonologicalWord> = Vec::new();
    let mut pending: Vec<usize> = Vec::new();
    for (i, p) in prosodies.iter().enumerate() {
        match p {
            Prosody::Proclitic => pending.push(i),
            Prosody::Tonic => {
                out.push(PhonologicalWord { host: i, proclitics: std::mem::take(&mut pending), enclitics: Vec::new() });
            }
            Prosody::Enclitic => match out.last_mut() {
                Some(unit) if pending.is_empty() => unit.enclitics.push(i),
                _ => out.push(PhonologicalWord { host: i, proclitics: std::mem::take(&mut pending), enclitics: Vec::new() }),
            },
        }
    }
    for i in pending {
        out.push(PhonologicalWord { host: i, proclitics: Vec::new(), enclitics: Vec::new() });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form::Form;
    use crate::grammar::Recension;

    #[test]
    fn the_unit_decides_the_final_varia() {
        let syn = Recension::Synodal;
        let zemlja = Form::from_print("землѧ̀");
        assert_eq!(zemlja.print(syn), "землѧ̀");
        assert_eq!(zemlja.print_unit(syn, &["же"]), "землѧ́же");
        // the jer before the enclitic (Synodal only), the print's own varia kept
        assert_eq!(Form::from_print("и҆̀хъ").print_unit(syn, &["же"]), "и҆̀хже");
        assert_eq!(Form::from_print("тѣ́мъ").print_unit(syn, &["же"]), "тѣ́мже");
        assert_eq!(Form::unaccented("имъ").print_unit(Recension::OldChurchSlavonic, &["же"]), "имъже");
        // a stem-stressed host is unchanged but for the enclitic
        assert_eq!(Form::from_print("бла́го").print_unit(syn, &["же"]), "бла́гоже");
        // the enclitic written apart: the host keeps the unit's oxia
        assert_eq!(zemlja.print_hosting(syn), "землѧ́");
        assert_eq!(Form::from_print("бла́го").print_hosting(syn), "бла́го");
        assert_eq!(Form::from_print("и҆̀хъ").print_hosting(syn), "и҆̀хъ");
    }

    #[test]
    fn grouping() {
        use Prosody::*;
        let units = words(&[Proclitic, Tonic, Enclitic, Tonic, Enclitic, Enclitic, Proclitic]);
        assert_eq!(units.len(), 3);
        assert_eq!(units[0], PhonologicalWord { host: 1, proclitics: vec![0], enclitics: vec![2] });
        assert_eq!(units[1], PhonologicalWord { host: 3, proclitics: vec![], enclitics: vec![4, 5] });
        assert_eq!(units[2].host, 6);
    }
}
