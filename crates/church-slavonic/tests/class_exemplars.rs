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

/// The Old Church Slavonic tables: the regular paradigms of the grammars
/// (the 1.x `regular_rules_golden`), one lexeme line per exemplar.
#[test]
fn old_church_slavonic() {
    const OCS: Recension = Recension::OldChurchSlavonic;
    let ocs = |line: &str, pos: Pos| -> Lexeme {
        let text = format!("id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n{line}\n");
        church_slavonic::lexicon::parse_in(&text, pos, OCS).expect("parses").remove(0)
    };
    let p = |l: &Lexeme, cell: &str| -> String {
        let cell = Cell::parse(l.pos, cell).unwrap_or_else(|| panic!("cell {cell}"));
        l.inflect(cell).unwrap_or_else(|| panic!("{}: no cell {}", l.id, cell.name())).print(OCS)
    };
    let rab = ocs("рабъ.n\tрабъ\tn\tm\t-\to:ъ:-\t-\t-\t-\t-\tK:o-stem\t-", Pos::Noun);
    assert_eq!(p(&rab, "gen.sg"), "раба");
    assert_eq!(p(&rab, "dat.sg"), "рабоу");
    assert_eq!(p(&rab, "loc.sg"), "рабѣ");
    assert_eq!(p(&rab, "voc.sg"), "рабе");
    assert_eq!(p(&rab, "nom.pl"), "раби");
    assert_eq!(p(&rab, "gen.pl"), "рабъ");
    assert_eq!(p(&rab, "acc.pl"), "рабꙑ");
    assert_eq!(p(&rab, "loc.pl"), "рабѣхъ");
    let zena = ocs("жена.n\tжена\tn\tf\t-\ta:а:-\t-\t-\t-\t-\tK:a-stem\t-", Pos::Noun);
    assert_eq!(p(&zena, "gen.sg"), "женꙑ");
    assert_eq!(p(&zena, "acc.sg"), "женѫ");
    assert_eq!(p(&zena, "ins.sg"), "женоѭ");
    assert_eq!(p(&zena, "dat.pl"), "женамъ");
    let drug = ocs("дроугъ.n\tдроугъ\tn\tm\t-\tok:ъ:-\t-\t-\t-\t-\tK:o-stem\t-", Pos::Noun);
    assert_eq!(p(&drug, "loc.sg"), "дроуѕѣ", "the second palatalisation writes ѕ");
    assert_eq!(p(&drug, "voc.sg"), "дроуже");
    assert_eq!(p(&drug, "nom.pl"), "дроуѕи");
    let kost = ocs("кость.n\tкость\tn\tf\t-\ti:ь:-\t-\t-\t-\t-\tK:i-stem\t-", Pos::Noun);
    assert_eq!(p(&kost, "gen.sg"), "кости");
    assert_eq!(p(&kost, "ins.pl"), "костьми");
    let imya = ocs("имѧ.n\tимѧ\tn\tn\t-\tn:ѧ:-\t-\t-\t-\t-\tK:n-stem\t-", Pos::Noun);
    assert_eq!(p(&imya, "gen.sg"), "имене");
    assert_eq!(p(&imya, "nom.pl"), "имена");
    // the present stem is the class's derivation (Leskien's classes), no
    // stem on the lexeme line: пити's jer before j
    let piti = ocs("пити.v\tпити\tv\t-\t-\tV:III:jer\t-\t-\t-\t-\tK:-\t-", Pos::Verb);
    assert_eq!(p(&piti, "pres.1.sg"), "пьѭ");
    assert_eq!(p(&piti, "pres.3.sg"), "пьѥтъ");
    assert_eq!(p(&piti, "aor.1.sg"), "пихъ");
    assert_eq!(p(&piti, "lpart.m.sg"), "пилъ");
    assert_eq!(p(&piti, "inf"), "пити");
    // class IV: the first person iotates (л-epenthesis, щ/жд for the
    // dentals), the rest of the present is on the plain stem with -и-;
    // after a husher the iotated vowel is written plain (прошѫ, хождаахъ)
    let v = |line: &str| ocs(line, Pos::Verb);
    let ljubiti = v("любити.v\tлюбити\tv\t-\t-\tV:IV:i\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&ljubiti, "pres.1.sg"), "люблѭ");
    assert_eq!(p(&ljubiti, "pres.2.sg"), "любиши");
    assert_eq!(p(&ljubiti, "pres.3.pl"), "любѧтъ");
    assert_eq!(p(&ljubiti, "impf.1.sg"), "люблꙗахъ");
    assert_eq!(p(&ljubiti, "part.pres.act.short.m.sg.nom"), "любѧ");
    assert_eq!(p(&ljubiti, "part.past.pass.short.m.sg.nom"), "любленъ");
    let prositi = v("просити.v\tпросити\tv\t-\t-\tV:IV:i\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&prositi, "pres.1.sg"), "прошѫ");
    assert_eq!(p(&prositi, "pres.3.sg"), "проситъ");
    assert_eq!(p(&prositi, "impf.1.sg"), "прошаахъ");
    let xoditi = v("ходити.v\tходити\tv\t-\t-\tV:IV:i\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&xoditi, "pres.1.sg"), "хождѫ");
    assert_eq!(p(&xoditi, "part.past.pass.short.m.sg.nom"), "хожденъ");
    // class III with -j-: the whole present iotated
    let pisati = v("писати.v\tписати\tv\t-\t-\tV:III:j\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&pisati, "pres.1.sg"), "пишѫ");
    assert_eq!(p(&pisati, "pres.2.sg"), "пишеши");
    assert_eq!(p(&pisati, "impv.2.sg"), "пиши");
    assert_eq!(p(&pisati, "aor.1.sg"), "писахъ");
    let glagolati = v("глаголати.v\tглаголати\tv\t-\t-\tV:III:j\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&glagolati, "pres.1.sg"), "глаголѭ");
    assert_eq!(p(&glagolati, "pres.3.sg"), "глаголѥтъ");
    // class III with -aje-, and -ova-
    let delati = v("дѣлати.v\tдѣлати\tv\t-\t-\tV:III:aje\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&delati, "pres.1.sg"), "дѣлаѭ");
    assert_eq!(p(&delati, "pres.3.sg"), "дѣлаѥтъ");
    let verovati = v("вѣровати.v\tвѣровати\tv\t-\t-\tV:III:ov\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&verovati, "pres.1.sg"), "вѣроуѭ"); // the OCS print writes ꙋ as оу
    assert_eq!(p(&verovati, "aor.1.sg"), "вѣровахъ");
    // class I: the consonant stems, the velars with the first
    // palatalisation in the present and the second in the imperative
    let nesti = v("нести.v\tнести\tv\t-\t-\tV:I:C\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&nesti, "pres.1.sg"), "несѫ");
    assert_eq!(p(&nesti, "pres.2.sg"), "несеши");
    let resti = v("рещи.v\tрещи\tv\t-\t-\tV:I:к\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&resti, "pres.1.sg"), "рекѫ");
    assert_eq!(p(&resti, "pres.2.sg"), "речеши");
    assert_eq!(p(&resti, "impv.2.sg"), "реци");
    assert_eq!(p(&resti, "part.pres.act.short.m.sg.nom"), "рекꙑ");
    let mosti = v("мощи.v\tмощи\tv\t-\t-\tV:I:г\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&mosti, "pres.1.sg"), "могѫ");
    assert_eq!(p(&mosti, "pres.3.sg"), "можетъ");
    assert_eq!(p(&mosti, "impv.2.sg"), "моѕи");
    // the jer grade of the root in the imperative (рьци, not реци) is a
    // lexical fact: an override on the lexeme line
    let resti = v("рещи.v\tрещи\tv\t-\t-\tV:I:к\t-\t-\timpv.2.sg=рьци\t-\tK:-\t-");
    assert_eq!(p(&resti, "impv.2.sg"), "рьци");
    let gresti = v("грѧсти.v\tгрѧсти\tv\t-\t-\tV:I:д\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&gresti, "pres.1.sg"), "грѧдѫ");
    assert_eq!(p(&gresti, "inf"), "грѧсти");
    // class II
    let dvignoti = v("двигнѫти.v\tдвигнѫти\tv\t-\t-\tV:II\t-\t-\t-\t-\tK:-\t-");
    assert_eq!(p(&dvignoti, "pres.1.sg"), "двигнѫ");
    assert_eq!(p(&dvignoti, "pres.2.sg"), "двигнеши");
    assert_eq!(p(&dvignoti, "inf"), "двигнѫти");
    let tu = ocs("тъ.pron\tтъ\tpron\tm\t-\tPA1\t-\t-\t-\t-\tK:-\t-", Pos::Pronoun);
    assert_eq!(p(&tu, "m.sg.gen"), "того");
    assert_eq!(p(&tu, "m.sg.ins"), "тѣмь");
    assert_eq!(p(&tu, "f.sg.acc"), "тѫ");
    assert_eq!(p(&tu, "m.pl.gen"), "тѣхъ");
    let az = ocs("азъ.pron\tазъ\tpron\t-\t-\tPPja\t-\t1=мен;2=мън;3=м\t-\t-\tU:\t-", Pos::Pronoun);
    assert_eq!(p(&az, "1.sg.gen"), "мене");
    assert_eq!(p(&az, "1.sg.dat"), "мънѣ");
    assert_eq!(p(&az, "clit.1.sg.acc"), "мѧ");
    let ize = ocs("иже.pron\tиже\tpron\tm\t-\tPPize\t-\tencl=же\t-\t-\tK:-\t-", Pos::Pronoun);
    assert_eq!(p(&ize, "m.sg.gen"), "ѥгоже");
    assert_eq!(p(&ize, "f.sg.nom"), "ꙗже");
    assert_eq!(p(&ize, "m.pl.dat"), "имъже");
}

