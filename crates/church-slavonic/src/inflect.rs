//! The four stages composed: a lexeme and a cell give a [`Form`]. An
//! override answers first (an attested print form the class + stress do
//! not produce); otherwise the class gives the letters and the stress
//! paradigm the position. There is no further fallback: a cell the class
//! does not declare answers `None`.

use crate::cell::Cell;
use crate::form::Form;
use crate::lexicon::Lexeme;
use crate::orthography::is_vowel_letter;
use crate::paradigm::{Class, Letters, Subject, table_of};
use crate::stress::{StressSpec, resolve_in};

impl Lexeme {
    /// The lemma's letters (marks stripped) and stressed vowel.
    pub fn lemma_form(&self) -> Form {
        Form::from_print(&self.lemma)
    }

    /// The class table row, if the class is known.
    pub fn class(&self) -> Option<&'static Class> {
        table_of(self.pos, self.recension).get(&self.class)
    }

    /// The stress paradigm; a malformed column is a lexicon error.
    pub fn stress_spec(&self) -> Option<StressSpec> {
        StressSpec::parse(&self.stress, self.pos)
            .unwrap_or_else(|e| panic!("{}: stress column: {e}", self.id))
    }

    fn compose(&self, cell: Cell, letters: &Letters, spec: Option<&StressSpec>, lemma_stress: Option<u8>) -> Form {
        // a solid enclitic's vowels never carry the word's stress
        // (возда́стсѧ, блюсти́сѧ): the ending's vowel count stops before it
        let total = letters.letters.chars().filter(|c| is_vowel_letter(*c)).count().saturating_sub(usize::from(letters.tail_vowels));
        let stress = spec.and_then(|s| resolve_in(s.place(cell), lemma_stress, letters.pre_vowels, letters.stem_vowels, total));
        Form { letters: letters.letters.clone(), stress, number_mark: letters.mark, mark_skip: letters.tail_vowels, varia: false, kamora: false }
    }

    /// The class's letters for a cell (the enclitic, if any,
    /// appended by the class).
    fn class_letters(&self, class: &Class, cell: Cell, lemma: &Form) -> Vec<Letters> {
        let subject = Subject { lemma: &lemma.letters, animate: self.animate, stems: &self.stems };
        class.letters(cell, &subject)
    }

    /// The primary form of `cell`: the override, else class + stress.
    /// `None` when the class does not declare the cell.
    pub fn inflect(&self, cell: impl Into<Cell>) -> Option<Form> {
        let cell = cell.into();
        if let Some((_, printed)) = self.overrides.iter().find(|(c, _)| *c == cell) {
            return Some(Form::from_print(printed));
        }
        if self.pos == crate::cell::Pos::Closed {
            return (cell == Cell::Word).then(|| Form::from_print(&self.lemma));
        }
        let class = self.class()?;
        let lemma = self.lemma_form();
        let letters = self.class_letters(class, cell, &lemma);
        let first = letters.first()?;
        Some(self.compose(cell, first, self.stress_spec().as_ref(), lemma.stress))
    }

    /// Every form of `cell`, primary first: the override or the class's
    /// primary, the class's other alternatives, then the lexeme's
    /// variants. Duplicates (by print) removed.
    pub fn forms(&self, cell: impl Into<Cell>) -> Vec<Form> {
        let cell = cell.into();
        let lemma = self.lemma_form();
        let class = self.class();
        let stems = class.map(|c| c.stems_of(&Subject { lemma: &lemma.letters, animate: self.animate, stems: &self.stems }));
        self.forms_with(cell, &lemma, class, stems.as_ref(), self.stress_spec().as_ref()).into_iter().map(|(f, _)| f).collect()
    }

    /// Every cell's forms with their prints (in the lexeme's recension), the
    /// lexeme's stems and stress paradigm computed once: what the analyzer's
    /// index walks.
    pub fn all_forms(&self) -> Vec<(Cell, Vec<(Form, String)>)> {
        let lemma = self.lemma_form();
        let class = self.class();
        let stems = class.map(|c| c.stems_of(&Subject { lemma: &lemma.letters, animate: self.animate, stems: &self.stems }));
        let spec = self.stress_spec();
        self.cells().into_iter().map(|cell| (cell, self.forms_with(cell, &lemma, class, stems.as_ref(), spec.as_ref()))).collect()
    }

    fn forms_with(
        &self,
        cell: Cell,
        lemma: &Form,
        class: Option<&'static Class>,
        stems: Option<&std::collections::HashMap<u8, String>>,
        spec: Option<&StressSpec>,
    ) -> Vec<(Form, String)> {
        let recension = self.recension;
        let mut out: Vec<(Form, String)> = Vec::new();
        let mut push = |f: Form| {
            let print = f.print(recension);
            if !out.iter().any(|(_, p)| *p == print) {
                out.push((f, print));
            }
        };
        if let Some((_, printed)) = self.overrides.iter().find(|(c, _)| *c == cell) {
            push(Form::from_print(printed));
        }
        if self.pos == crate::cell::Pos::Closed && cell == Cell::Word {
            push(Form::from_print(&self.lemma));
        }
        if let (Some(class), Some(stems)) = (class, stems) {
            let subject = Subject { lemma: &lemma.letters, animate: self.animate, stems: &self.stems };
            for letters in class.letters_with(cell, &subject, stems) {
                push(self.compose(cell, &letters, spec, lemma.stress));
            }
        }
        for (_, variants) in self.variants.iter().filter(|(c, _)| *c == cell) {
            for v in variants {
                push(Form::from_print(v));
            }
        }
        out
    }

    /// The whole paradigm in the class's cell order (overrides included);
    /// a closed-class word's one cell.
    pub fn paradigm(&self) -> Vec<(Cell, Form)> {
        if self.pos == crate::cell::Pos::Closed {
            return self.inflect(Cell::Word).map(|f| vec![(Cell::Word, f)]).unwrap_or_default();
        }
        let Some(class) = self.class() else { return Vec::new() };
        class.order.iter().filter_map(|c| self.inflect(*c).map(|f| (*c, f))).collect()
    }

    /// The cells the lexeme declares, in order.
    pub fn cells(&self) -> Vec<Cell> {
        if self.pos == crate::cell::Pos::Closed {
            return vec![Cell::Word];
        }
        self.class().map(|c| c.order.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::{NounCell, Pos};
    use crate::grammar::{Case, Number, Recension};
    use crate::lexicon::parse;
    use unicode_normalization::UnicodeNormalization;

    const SYN: Recension = Recension::Synodal;

    fn nfc(s: &str) -> String {
        s.nfc().collect()
    }

    fn lexeme(line: &str) -> crate::lexicon::Lexeme {
        let text = format!("id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n{line}\n");
        parse(&text, Pos::Noun).expect("parses").remove(0)
    }

    fn cell(s: &str) -> NounCell {
        NounCell::parse(s).expect("cell")
    }

    #[test]
    fn rab_is_one_line() {
        let rab = lexeme("рабъ.n\tра́бъ\tn\tm\tanim\tN1t\tb{voc.sg=S}\t-\t-\tgen.pl=ра̑бъ\tP:N1t\t-");
        let p = |c: &str| rab.inflect(cell(c)).expect("cell").print(SYN);
        assert_eq!(p("nom.sg"), nfc("ра́бъ"));
        assert_eq!(p("gen.sg"), nfc("раба̀"));
        assert_eq!(p("dat.sg"), nfc("рабꙋ̀"));
        assert_eq!(p("acc.sg"), nfc("раба̀"));
        assert_eq!(p("ins.sg"), nfc("рабо́мъ"));
        assert_eq!(p("loc.sg"), nfc("рабѣ̀"));
        assert_eq!(p("voc.sg"), nfc("ра́бе"));
        assert_eq!(p("nom.pl"), nfc("рабѝ"));
        assert_eq!(p("gen.pl"), nfc("рабѡ́въ"));
        assert_eq!(p("dat.pl"), nfc("рабѡ́мъ"));
        assert_eq!(p("acc.pl"), nfc("рабы̀"), "the print's plural accusative (Polyakov: рабы́ 159 tagged acc alone)");
        assert_eq!(p("loc.pl"), nfc("рабѣ́хъ"));
        assert_eq!(p("nom.du"), nfc("раба̑"));
        assert_eq!(p("dat.du"), nfc("рабо́ма"));
        let gen_pl: Vec<String> = rab.forms(cell("gen.pl")).iter().map(|f| f.print(SYN)).collect();
        assert_eq!(gen_pl, [nfc("рабѡ́въ"), nfc("ра̑бъ")]);
        assert_eq!(rab.paradigm().len(), 21);
    }

    #[test]
    fn stem_stress_fleeting_vowels_and_overrides() {
        let grad = lexeme("градъ.n\tгра́дъ\tn\tm\tinan\tN1t\ta\t-\t-\t-\tP:N1t\t-");
        let p = |l: &crate::lexicon::Lexeme, c: &str| l.inflect(cell(c)).expect("cell").print(SYN);
        assert_eq!(p(&grad, "gen.sg"), nfc("гра́да"));
        assert_eq!(p(&grad, "acc.sg"), nfc("гра́дъ"));
        assert_eq!(p(&grad, "dat.pl"), nfc("гра́дѡмъ"));
        assert_eq!(p(&grad, "acc.pl"), nfc("гра́ды"));
        assert_eq!(p(&grad, "nom.du"), nfc("гра̑да"));
        let otec = lexeme("ѻтецъ.n\tѻ҆те́цъ\tn\tm\tanim\tN1c*\tb\t-\tvoc.sg=ѻ҆́тче\t-\tP:N1c*\t-");
        assert_eq!(p(&otec, "nom.sg"), nfc("ѻ҆те́цъ"));
        assert_eq!(p(&otec, "gen.sg"), nfc("ѻ҆тца̀"));
        assert_eq!(p(&otec, "voc.sg"), nfc("ѻ҆́тче"), "the override");
        assert_eq!(p(&otec, "gen.pl"), nfc("ѻ҆тцє́въ"));
        let syn = lexeme("сынъ.n\tсы́нъ\tn\tm\tanim\tN1t\ta{pl=E;nom.pl=S}\t-\tnom.pl=сы́нове\t-\tP:N1t\t-");
        assert_eq!(p(&syn, "gen.sg"), nfc("сы́на"));
        assert_eq!(p(&syn, "gen.pl"), nfc("сынѡ́въ"));
        assert_eq!(p(&syn, "nom.pl"), nfc("сы́нове"));
        let imya = lexeme("имѧ.n\tи҆́мѧ\tn\tn\tinan\tN5en\ta{pl=E}\t-\t-\t-\tP:N5en\t-");
        assert_eq!(p(&imya, "nom.sg"), nfc("и҆́мѧ"));
        assert_eq!(p(&imya, "gen.sg"), nfc("и҆́мене"));
        assert_eq!(p(&imya, "nom.pl"), nfc("и҆мена̀"));
        assert_eq!(p(&imya, "gen.pl"), nfc("и҆ме́нъ"), "no ending vowel: the last stem vowel");
    }

    #[test]
    fn the_enclitic_never_carries_the_stress() {
        let text = "id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n\
            воздатисѧ.v\tвозда́тисѧ\tv\t-\t-\tVdat\tb\tencl=сѧ\t-\t-\tP:Vdat\t-\n\
            блюстисѧ.v\tблюсти́сѧ\tv\t-\t-\tV14d\tb\tencl=сѧ\t-\t-\tP:V14d\t-\n";
        let verbs = parse(&text, Pos::Verb).expect("parses");
        let p = |l: &crate::lexicon::Lexeme, c: &str| l.inflect(crate::cell::Cell::parse(Pos::Verb, c).expect("cell")).expect("cell").print(SYN);
        // no ending vowel: the last stem vowel, not the enclitic's
        assert_eq!(p(&verbs[0], "pres.3.sg"), nfc("возда́стсѧ"));
        assert_eq!(p(&verbs[0], "pres.1.sg"), nfc("возда́мсѧ"));
        assert_eq!(p(&verbs[1], "inf"), nfc("блюсти́сѧ"));
        // an ending with a vowel keeps the ending stress
        assert_eq!(p(&verbs[1], "pres.3.sg"), nfc("блюде́тсѧ"));
    }

    #[test]
    fn an_unknown_class_answers_nothing() {
        let x = lexeme("x.n\tх\tn\tm\tanim\tNOPE\ta\t-\t-\t-\t-\t-");
        assert!(x.inflect(cell("nom.sg")).is_none());
        assert!(x.paradigm().is_empty());
        assert_eq!(x.inflect(NounCell::new(Case::Nominative, Number::Singular)), None);
    }
}
