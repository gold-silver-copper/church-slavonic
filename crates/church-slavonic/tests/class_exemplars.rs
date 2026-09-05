//! The legend's exemplars through the class tables: one lexeme line per
//! exemplar, the print forms the tables must produce.

use church_slavonic::{Cell, Lexeme, Pos, Recension};
use unicode_normalization::UnicodeNormalization;

const SYN: Recension = Recension::Synodal;

fn nfc(s: &str) -> String {
    s.nfc().collect()
}

fn lexeme(line: &str, pos: Pos) -> Lexeme {
    let text = format!("id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n{line}\n");
    church_slavonic::lexicon::parse(&text, pos).expect("parses").remove(0)
}

fn print(l: &Lexeme, cell: &str) -> String {
    let cell = Cell::parse(l.pos, cell).unwrap_or_else(|| panic!("cell {cell}"));
    l.inflect(cell).unwrap_or_else(|| panic!("{}: no cell {}", l.id, cell.name())).print(SYN)
}

#[test]
fn adjectives() {
    let mudr = lexeme("мꙋдрый.a\tмꙋ́дрый\ta\t-\t-\tA1t\ta\t-\t-\t-\tP:A1t\t-", Pos::Adjective);
    assert_eq!(print(&mudr, "long.pos.m.sg.nom"), nfc("мꙋ́дрый"));
    assert_eq!(print(&mudr, "long.pos.m.sg.gen"), nfc("мꙋ́драгѡ"));
    // the masculine accusative: the nominative-shaped reading is the
    // measured majority; the animate -аго stays reachable
    assert_eq!(print(&mudr, "long.pos.m.sg.acc"), nfc("мꙋ́дрый"));
    let acc: Vec<String> = mudr.forms(Cell::parse(Pos::Adjective, "long.pos.m.sg.acc").expect("cell")).iter().map(|f| f.print(SYN)).collect();
    assert!(acc.contains(&nfc("мꙋ́драго")), "{acc:?}");
    assert_eq!(print(&mudr, "long.pos.f.sg.nom"), nfc("мꙋ́драѧ"));
    assert_eq!(print(&mudr, "long.pos.m.pl.nom"), nfc("мꙋ́дрїи"), "the print's ї before a vowel");
    assert_eq!(print(&mudr, "long.pos.f.pl.nom"), nfc("мꙋ̑дрыѧ"));
    assert_eq!(print(&mudr, "short.pos.m.sg.nom"), nfc("мꙋ́дръ"));
    assert_eq!(print(&mudr, "short.pos.f.sg.nom"), nfc("мꙋ́дра"));
    assert_eq!(print(&mudr, "short.pos.m.sg.ins"), nfc("мꙋ́дрымъ"), "the short instrumental is the long form");
    assert_eq!(print(&mudr, "short.pos.m.sg.voc"), nfc("мꙋ́дре"));
    assert_eq!(print(&mudr, "long.comp.m.sg.nom"), nfc("мꙋ́дрѣйшїй"));
    assert_eq!(print(&mudr, "short.comp.m.sg.nom"), nfc("мꙋ́дрѣй"));
    let bez = lexeme("беззаконный.a\tбеззако́нный\ta\t-\t-\tA1t*\ta\t-\t-\t-\tP:A1t*\t-", Pos::Adjective);
    assert_eq!(print(&bez, "short.pos.m.sg.nom"), nfc("беззако́ненъ"));
    assert_eq!(print(&bez, "long.pos.m.sg.gen"), nfc("беззако́ннагѡ"));
    let chis = lexeme("безчисленный.a\tбезчи́сленный\ta\t-\t-\tA1n*\ta\t-\t-\t-\tP:A1n*\t-", Pos::Adjective);
    assert_eq!(print(&chis, "short.pos.m.sg.nom"), nfc("безчи́сленъ"));
    let velik = lexeme("великій.a\tвели́кій\ta\t-\t-\tA1k\ta{long.comp=L;short.comp=L}\t-\t-\t-\tP:A1k\t-", Pos::Adjective);
    assert_eq!(print(&velik, "long.pos.m.sg.loc"), nfc("вели́цѣмъ"));
    assert_eq!(print(&velik, "long.pos.m.pl.nom"), nfc("вели́цыи"));
    assert_eq!(print(&velik, "short.pos.m.pl.nom"), nfc("вели́цы"));
    assert_eq!(print(&velik, "long.comp.m.sg.nom"), nfc("велича́йшїй"), "the comparative stresses its suffix (L)");
    let david = lexeme("давідовъ.a\tдаві́довъ\ta\t-\t-\tA2t\ta\t-\t-\t-\tP:A2t\t-", Pos::Adjective);
    assert_eq!(print(&david, "short.pos.m.sg.nom"), nfc("даві́довъ"));
    assert_eq!(print(&david, "short.pos.f.sg.nom"), nfc("даві́дова"));
    assert_eq!(print(&david, "short.pos.m.sg.ins"), nfc("даві́довымъ"));
    let bozhij = lexeme("божій.a\tбо́жій\ta\t-\t-\tA2i\ta\t-\t-\t-\tP:A2i\t-", Pos::Adjective);
    assert_eq!(print(&bozhij, "short.pos.m.sg.nom"), nfc("бо́жїй"));
    assert_eq!(print(&bozhij, "short.pos.f.sg.nom"), nfc("бо́жїѧ"));
    assert_eq!(print(&bozhij, "short.pos.m.sg.gen"), nfc("бо́жїѧ"));
    let nast = lexeme("настоѧщій.a\tнастоѧ́щій\ta\t-\t-\tA1s\ta\t-\t-\t-\tP:A1s\t-", Pos::Adjective);
    assert_eq!(print(&nast, "long.pos.m.sg.gen"), nfc("настоѧ́щагѡ"));
    assert_eq!(print(&nast, "long.pos.f.pl.nom"), nfc("настоѧ́щыѧ"));
}

#[test]
fn verbs() {
    let tvor = lexeme("творити.v\tтвори́ти\tv\t-\t-\tV21n\tb\t-\t-\t-\tP:V21n\t-", Pos::Verb);
    assert_eq!(print(&tvor, "pres.1.sg"), nfc("творю̀"));
    assert_eq!(print(&tvor, "pres.2.sg"), nfc("твори́ши"));
    assert_eq!(print(&tvor, "pres.3.pl"), nfc("творѧ́тъ"));
    assert_eq!(print(&tvor, "impf.3.sg"), nfc("творѧ́ше"));
    assert_eq!(print(&tvor, "aor.1.sg"), nfc("твори́хъ"));
    assert_eq!(print(&tvor, "aor.3.pl"), nfc("твори́ша"));
    assert_eq!(print(&tvor, "impv.2.sg"), nfc("творѝ"));
    assert_eq!(print(&tvor, "impv.2.pl"), nfc("твори́те"));
    assert_eq!(print(&tvor, "inf"), nfc("твори́ти"));
    assert_eq!(print(&tvor, "lpart.m.sg"), nfc("твори́лъ"));
    assert_eq!(print(&tvor, "lpart.f.sg"), nfc("твори́ла"));
    assert_eq!(print(&tvor, "part.pres.act.short.m.sg.nom"), nfc("творѧ̀"));
    assert_eq!(print(&tvor, "part.pres.act.long.m.sg.nom"), nfc("творѧ́й"));
    assert_eq!(print(&tvor, "part.pres.act.long.m.sg.gen"), nfc("творѧ́щагѡ"));
    assert_eq!(print(&tvor, "part.pres.act.short.f.sg.nom"), nfc("творѧ́щи"));
    assert_eq!(print(&tvor, "part.pres.act.short.n.sg.nom"), nfc("творѧ̀"));
    assert_eq!(print(&tvor, "part.pres.pass.short.m.sg.nom"), nfc("твори́мъ"));
    assert_eq!(print(&tvor, "part.pres.act.short.m.sg.acc"), nfc("творѧ́ща"));
    assert_eq!(print(&tvor, "part.pres.act.short.m.pl.nom"), nfc("творѧ́ще"));
    assert_eq!(print(&tvor, "part.past.pass.short.m.pl.gen"), nfc("творє́нныхъ"));
    assert_eq!(print(&tvor, "part.pres.pass.long.m.sg.nom"), nfc("твори́мый"));
    assert_eq!(print(&tvor, "part.past.act.short.m.sg.nom"), nfc("твори́въ"));
    assert_eq!(print(&tvor, "part.past.act.long.m.sg.nom"), nfc("твори́вый"));
    assert_eq!(print(&tvor, "part.past.act.long.m.sg.gen"), nfc("твори́вшагѡ"));
    assert_eq!(print(&tvor, "part.past.pass.short.m.sg.nom"), nfc("творе́нъ"));
    assert_eq!(print(&tvor, "part.past.pass.long.m.sg.nom"), nfc("творе́нный"));
    let lub = lexeme("любити.v\tлюби́ти\tv\t-\t-\tV21p\tb\t-\t-\t-\tP:V21p\t-", Pos::Verb);
    assert_eq!(print(&lub, "pres.1.sg"), nfc("люблю̀"));
    assert_eq!(print(&lub, "pres.2.sg"), nfc("люби́ши"));
    let rod = lexeme("родити.v\tроди́ти\tv\t-\t-\tV21t\tb\t-\t-\t-\tP:V21t\t-", Pos::Verb);
    assert_eq!(print(&rod, "pres.1.sg"), nfc("рождꙋ̀"));
    assert_eq!(print(&rod, "part.past.pass.short.m.sg.nom"), nfc("рожде́нъ"));
    let dela = lexeme("дѣлати.v\tдѣ́лати\tv\t-\t-\tV11a\ta\t-\t-\t-\tP:V11a\t-", Pos::Verb);
    assert_eq!(print(&dela, "pres.1.sg"), nfc("дѣ́лаю"));
    assert_eq!(print(&dela, "pres.3.sg"), nfc("дѣ́лаетъ"));
    assert_eq!(print(&dela, "aor.3.sg"), nfc("дѣ́ла"));
    assert_eq!(print(&dela, "impv.2.sg"), nfc("дѣ́лай"));
    assert_eq!(print(&dela, "part.pres.act.long.m.sg.nom"), nfc("дѣ́лаѧй"));
    assert_eq!(print(&dela, "part.pres.act.long.f.sg.nom"), nfc("дѣ́лающаѧ"));
    let treb = lexeme("требовати.v\tтре́бовати\tv\t-\t-\tV12ov\ta\t-\t-\t-\tP:V12ov\t-", Pos::Verb);
    assert_eq!(print(&treb, "pres.1.sg"), nfc("тре́бꙋю"));
    assert_eq!(print(&treb, "aor.1.sg"), nfc("тре́бовахъ"));
    let nes = lexeme("нести.v\tнестѝ\tv\t-\t-\tV14z\tb\t-\t-\t-\tP:V14z\t-", Pos::Verb);
    assert_eq!(print(&nes, "pres.1.sg"), nfc("несꙋ̀"));
    assert_eq!(print(&nes, "aor.1.sg"), nfc("несо́хъ"));
    assert_eq!(print(&nes, "aor.3.sg"), nfc("несѐ"));
    let pek = lexeme("пещи.v\tпещѝ\tv\t-\t-\tV14k\tb\t-\t-\t-\tP:V14k\t-", Pos::Verb);
    assert_eq!(print(&pek, "pres.1.sg"), nfc("пекꙋ̀"));
    assert_eq!(print(&pek, "pres.2.sg"), nfc("пече́ши"));
    assert_eq!(print(&pek, "impv.2.sg"), nfc("пецы̀"));
    assert_eq!(print(&pek, "part.past.pass.short.m.sg.nom"), nfc("пече́нъ"));
    let vzyat = lexeme("взѧти.v\tвзѧ́ти\tv\t-\t-\tV15n\tb\t-\t-\t-\tP:V15n\t-", Pos::Verb);
    assert_eq!(print(&vzyat, "aor.1.sg"), nfc("взѧ́хъ"));
    assert_eq!(print(&vzyat, "pres.1.sg"), nfc("взнꙋ̀"), "the class rule; the lexeme says stems=2=возм");
}

/// The pronominal stress is per cell in the print (тогѡ̀ but то́ю): the
/// letters are the class's, the stress the lexeme line's.
fn key(l: &Lexeme, cell: &str) -> String {
    church_slavonic::orthography::comparison_key(&print(l, cell))
}

#[test]
fn pronominal() {
    let toj = lexeme("той.pron\tто́й\tpron\tm\t-\tPA1t\tb\t-\t-\t-\tP:PA1t\t-", Pos::Pronoun);
    assert_eq!(print(&toj, "m.sg.nom"), nfc("то́й"));
    assert_eq!(key(&toj, "m.sg.gen"), church_slavonic::orthography::comparison_key("тогѡ̀"));
    assert_eq!(key(&toj, "f.sg.nom"), church_slavonic::orthography::comparison_key("та́ѧ"));
    assert_eq!(key(&toj, "m.pl.nom"), church_slavonic::orthography::comparison_key("ті́и"));
    let ves = lexeme("весь.pron\tве́сь\tpron\tm\t-\tPA1j*\tb\t-\t-\t-\tP:PA1j*\t-", Pos::Pronoun);
    assert_eq!(print(&ves, "m.sg.nom"), nfc("ве́сь"));
    assert_eq!(key(&ves, "m.sg.gen"), church_slavonic::orthography::comparison_key("всегѡ̀"));
    let az = lexeme("азъ.pron\tа҆́зъ\tpron\t-\t-\tPPja\tb\t1=мен;2=мн;3=м\t-\t-\tP:PPja\t-", Pos::Pronoun);
    assert_eq!(print(&az, "1.sg.gen"), nfc("менє̀"));
    assert_eq!(print(&az, "1.sg.acc"), nfc("менѐ"));
    assert_eq!(print(&az, "1.sg.dat"), nfc("мнѣ̀"));
    assert_eq!(print(&az, "clit.1.sg.acc"), nfc("мѧ̀"));
}

#[test]
fn reflexive() {
    let v = lexeme("взалкатисѧ.v\tвзалка́тисѧ\tv\t-\t-\tV12t\tb\tencl=сѧ\t-\t-\tP:V12t\t-", Pos::Verb);
    assert_eq!(print(&v, "inf"), nfc("взалка́тисѧ"));
    assert_eq!(print(&v, "pres.1.sg"), nfc("взалчꙋ́сѧ"));
    assert_eq!(print(&v, "aor.3.sg"), nfc("взалка́сѧ"));
    let b = lexeme("боѧтисѧ.v\tбоѧ́тисѧ\tv\t-\t-\tV22a\tb\tencl=сѧ\t-\t-\tP:V22a\t-", Pos::Verb);
    assert_eq!(print(&b, "pres.1.sg"), nfc("бою́сѧ"));
    assert_eq!(print(&b, "pres.3.sg"), nfc("бои́тсѧ"));
    assert_eq!(print(&b, "aor.1.sg"), nfc("боѧ́хсѧ"));
    assert_eq!(print(&b, "lpart.m.sg"), nfc("боѧ́лсѧ"));
    assert_eq!(print(&b, "part.pres.act.long.m.pl.gen"), nfc("боѧ́щихсѧ"));
}
