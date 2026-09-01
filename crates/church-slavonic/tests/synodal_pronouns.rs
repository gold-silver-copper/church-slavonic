//! The v1.2 ledger: the Synodal pronoun program's exact expected outputs,
//! each justified by a quoted line of the pinned Elizabethan Bible
//! (`data/witnesses.tsv`, verified by `cargo xtask check-witnesses`) or by
//! the Alypy grammar's paradigm (§47) where the Bible has no token to
//! offer. Part 1: the personal row arbitrated against the print.

use church_slavonic::*;

const SYN: Recension = Recension::Synodal;

fn pers(p: Person, n: Number, g: Gender, c: Case) -> &'static str {
    ChurchSlavonic::pronoun(&p, &n, &g, &c, &SYN)
}

#[test]
fn part_1_the_personal_row_reads_as_the_print() {
    use Case::*;
    use Gender::*;
    use Number::*;
    use Person::*;
    // Gen 19:13 «ꙗ҆́кѡ мы̀ погꙋблѧ́емъ мѣ́сто сїѐ» — Polyakov tags «ны̀» as
    // a nominative 2,531 times; the print's subject is мы̀.
    assert_eq!(pers(First, Plural, Masculine, Nominative), "мы̀");
    // Gen 3:10 «гла́съ слы́шахъ тебє̀ ходѧ́ща въ раѝ» (the grammar prints
    // the genitive as тебѐ; the corpus and the Bible spell є).
    assert_eq!(pers(Second, Singular, Masculine, Genitive), "тебє̀");
    // Gen 15:10 «Взѧ́ же ѻ҆́нъ всѧ̑ сїѧ̑» — the nominative is the ѻ҆́нъ
    // series; the anaphor's «и҆̀» is an accusative (below).
    assert_eq!(pers(Third, Singular, Masculine, Nominative), "ѻ҆́нъ");
    // Lev 20:17 «и҆ ѻ҆на̀ ѹ҆ви́дитъ срамотꙋ̀ є҆гѡ̀» — not the bundle's «ꙗ҆̀».
    assert_eq!(pers(Third, Singular, Feminine, Nominative), "ѻ҆на̀");
    // Alypy §47 (the Bible has no neuter-singular subject pronoun token).
    assert_eq!(pers(Third, Singular, Neuter, Nominative), "ѻ҆но̀");
    assert_eq!(pers(Third, Plural, Feminine, Nominative), "ѻ҆нѣ̀");
    // Gen 16:8 «И҆ речѐ є҆́й а҆́гг҃лъ гдⷭ҇ень» — the dative without a
    // preposition; «не́й» is the prepositional (locative) form.
    assert_eq!(pers(Third, Singular, Feminine, Dative), "є҆́й");
    assert_eq!(pers(Third, Singular, Feminine, Locative), "не́й");
    // Gen 9:1 «и҆ ѡ҆блада́йте є҆́ю».
    assert_eq!(pers(Third, Singular, Feminine, Instrumental), "є҆́ю");
    // Gen 1:17 «и҆ положѝ ѧ҆̀ бг҃ъ на тве́рди небе́снѣй» — the two lights.
    // The committed row spelled «ꙗ҆̀» here: Polyakov's civil «я́».
    assert_eq!(pers(Third, Dual, Neuter, Accusative), "ѧ҆̀");
    assert_eq!(pers(Third, Dual, Masculine, Accusative), "ѧ҆̀");
    assert_eq!(pers(Third, Dual, Feminine, Accusative), "ѧ҆̀");
    // Gen 9:1 «и҆ речѐ и҆̀мъ» / Gen 6:13 «а҆́зъ погꙋблю̀ и҆̀хъ» — the print
    // tells the dative and accusative from the genitive «и҆́хъ» by the
    // varia on the monosyllable; Alypy §47 prints the same.
    assert_eq!(pers(Third, Plural, Masculine, Dative), "и҆̀мъ");
    assert_eq!(pers(Third, Plural, Feminine, Dative), "и҆̀мъ");
    assert_eq!(pers(Third, Plural, Masculine, Accusative), "и҆̀хъ");
    assert_eq!(pers(Third, Plural, Masculine, Genitive), "и҆́хъ");
    // Gen 24:3 «а҆́зъ живꙋ̀ въ ни́хъ» / Gen 30:34 / Eph 2:10 — the locative
    // is the prepositional form; the bundle had put the genitive there.
    assert_eq!(pers(Third, Plural, Masculine, Locative), "ни́хъ");
    assert_eq!(pers(Third, Plural, Feminine, Locative), "ни́хъ");
    assert_eq!(pers(Third, Plural, Neuter, Locative), "ни́хъ");
    // The anaphor's short accusative and the dictionary's «ны̀» stay
    // reachable as variants; the civil-transliterated «ꙗ҆̀» is not a form
    // of its own (it differs from «ѧ҆̀» only in what civil «я» cannot
    // encode) and is stored nowhere.
    let reachable = |form: &str, p: Person, n: Number, g: Gender, c: Case| {
        (2..=16).any(|k| ChurchSlavonic::pronoun_sense(&format!("personal_{k}"), &p, &n, &g, &c, &SYN) == form)
    };
    assert!(!reachable("ꙗ҆̀", Third, Dual, Neuter, Accusative));
    assert!(reachable("и҆̀", Third, Singular, Masculine, Accusative));
    assert!(reachable("ны̀", First, Plural, Masculine, Accusative));
}

fn pn(lemma: &str, g: Gender, n: Number, c: Case) -> String {
    ChurchSlavonic::npron(lemma, &g, &n, &c, &SYN)
}

#[test]
fn part_2_the_non_personal_pronouns_read_as_the_print() {
    use Case::*;
    use Gender::*;
    use Number::*;
    // Gen 1:31 «И҆ ви́дѣ бг҃ъ всѧ̑, є҆ли̑ка сотворѝ» — the plural's kamora,
    // which the dictionary's bundle spells as the singular's всѧ̀.
    assert_eq!(pn("ве́сь", Neuter, Plural, Accusative), "всѧ̑");
    assert_eq!(pn("ве́сь", Masculine, Plural, Accusative), "всѧ̑");
    assert_eq!(pn("ве́сь", Masculine, Plural, Nominative), "всѝ");
    assert_eq!(pn("ве́сь", Masculine, Singular, Genitive), "всегѡ̀");
    assert_eq!(pn("ве́сь", Feminine, Singular, Nominative), "всѧ̀");
    // the possessives are the rule's throughout (no bare table row)
    assert_eq!(pn("мо́й", Feminine, Plural, Nominative), "моѧ̑");
    assert_eq!(pn("тво́й", Masculine, Singular, Genitive), "твоегѡ̀");
    assert_eq!(pn("сво́й", Feminine, Singular, Genitive), "своеѧ̀");
    assert_eq!(pn("сво́й", Masculine, Plural, Dative), "свои̑мъ");
    assert_eq!(pn("на́шъ", Neuter, Plural, Nominative), "на̑ша");
    // Gen 14:5 «цари̑ и҆̀же съ ни́мъ» / Ex 6:26 «и҆̀мже речѐ бг҃ъ» / Gen 1:21
    // «ꙗ҆̀же и҆зведо́ша во́ды»: the relative's plural varia.
    assert_eq!(pn("и҆́же", Masculine, Plural, Nominative), "и҆̀же");
    assert_eq!(pn("и҆́же", Masculine, Plural, Dative), "и҆̀мже");
    assert_eq!(pn("и҆́же", Feminine, Plural, Nominative), "ꙗ҆̀же");
    assert_eq!(pn("и҆́же", Masculine, Plural, Accusative), "ꙗ҆̀же");
    assert_eq!(pn("и҆́же", Masculine, Singular, Genitive), "є҆гѡ́же");
    assert_eq!(pn("и҆́же", Feminine, Singular, Nominative), "ꙗ҆́же");
    assert_eq!(pn("и҆́же", Neuter, Singular, Nominative), "є҆́же");
    // Gen 24:37 «въ ни́хже а҆́зъ ѡ҆бита́ю»
    assert_eq!(pn("и҆́же", Feminine, Plural, Locative), "ни́хже");
    // the interrogatives answer every gender and number the same
    assert_eq!(pn("кто̀", Feminine, Dual, Dative), "комꙋ̀");
    assert_eq!(pn("что̀", Masculine, Singular, Genitive), "чесѡ̀");
    assert_eq!(pn("никто́же", Masculine, Singular, Dative), "никомꙋ́же");
    // the same-series split: the long series is the adjective's
    assert_eq!(pn("всѧ́къ", Feminine, Singular, Accusative), "всѧ́кꙋ");
    assert_eq!(
        ChurchSlavonic::adj("всѧ́кій", &Genitive, &Singular, &Masculine, &Degree::Positive, &SYN),
        "всѧ́кагѡ"
    );
    // the inventory is enumerable
    let lemmas: Vec<&str> = ChurchSlavonic::lemmas(PartOfSpeech::NonPersonalPronoun, &SYN).collect();
    for l in ["ве́сь", "и҆́же", "то́й", "се́й", "на́шъ", "кто̀", "что̀", "всѧ́къ", "є҆ди́нъ"] {
        assert!(lemmas.contains(&l), "{l}");
    }
    assert!(!lemmas.contains(&"мъ"));
}
