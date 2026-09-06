//! The auto-lift: a verse token to an analyzed leaf through the 2.0
//! analyzer. A token lifts ONLY when its surface has exactly one EXACT
//! reading (the lexicon's own print, byte-for-byte), so the round-trip
//! invariant survives lifting by construction. Several readings are
//! recorded (`:amb <n>`) and the token kept verbatim, never guessed:
//! disambiguation by syntactic context is a later, separate design. No
//! reading and a word of the hand closed-class table gives `(f …)`.
//!
//! The titlo layer: a token under a titlo (гдⷭ҇а, бг҃ꙋ) is looked up in a
//! small secondary index — every cell of every `data/titlo.tsv` row's
//! lemma, abbreviated under the row — and lifts as `(abbr "гдⷭ҇" leaf)`.
//!
//! Apparatus tokens (anything containing `꙾` or `[`) stay verbatim
//! wholesale. Other tokens split into leading punctuation, core, and
//! trailing punctuation; a capitalized core is looked up decapitalized
//! and lifts under `(cap …)`.

use crate::sentence::node::Node;
use crate::cell::{Cell, CellSet};
use crate::{Lexicon, Recension};
use std::collections::HashMap;

/// The titlo index: abbreviated surface → (row prefix, id, cell, alt).
/// One expansion of a titlo-written surface: the abbreviated prefix, the
/// lexeme id, the cell, the alternative, and the row's full-prefix skeleton.
pub type TitloEntry = (String, String, Cell, usize, String);

pub struct TitloIndex {
    /// surface → its entries
    map: HashMap<String, Vec<TitloEntry>>,
}

impl TitloIndex {
    pub fn build(lexicon: &Lexicon) -> TitloIndex {
        let mut map: HashMap<String, Vec<TitloEntry>> = HashMap::new();
        for row in crate::titlo::rows() {
            // the row's lemma: every lexeme whose lemma prints as it
            let key = crate::orthography::comparison_key(row.lemma);
            for lexeme in lexicon.iter().filter(|l| crate::orthography::comparison_key(&l.lemma) == key) {
                for cell in lexeme.cells() {
                    for (alt, form) in lexeme.forms(cell).into_iter().enumerate() {
                        let full = form.print(lexicon.recension);
                        if let Some(abbreviated) = crate::titlo::abbreviate(&full, row) {
                            let entry = map.entry(abbreviated).or_default();
                            let item = (row.abbr.to_string(), lexeme.id.clone(), cell, alt, row.full.to_string());
                            if !entry.contains(&item) {
                                entry.push(item);
                            }
                        }
                    }
                }
            }
        }
        TitloIndex { map }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// The index's entries for an abbreviated surface.
    pub fn entries(&self, surface: &str) -> Option<&[TitloEntry]> {
        self.map.get(surface).map(Vec::as_slice)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// The cells of one lexeme that abbreviate to `surface` under the
    /// titlo row `prefix` (the abbreviation erases the accent that tells
    /// дꙋ́хъ from дꙋ̑хъ, so дх҃ъ is nom.sg|gen.pl|acc.pl).
    pub fn cells(&self, surface: &str, _prefix: &str, id: &str) -> Option<CellSet> {
        // every row of the lexeme that abbreviates to the surface counts
        // (4.1: the lifter reads one lexeme as one reading whatever the
        // row, нб҃са̀ under нб҃с/небес and нб҃/неб alike)
        let mut cells: Vec<Cell> = self.map.get(surface)?.iter().filter(|(_, i, _, _, _)| i == id).map(|(_, _, c, _, _)| *c).collect();
        cells.dedup();
        CellSet::new(cells)
    }
}

/// What one verse token became.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenFate {
    /// One lexeme, one cell.
    Analyzed,
    /// One lexeme, several cells the paradigm does not tell apart: the
    /// leaf carries the set (syncretism, not doubt).
    Underspecified,
    ClosedClass,
    Ambiguous,
    Verbatim,
    Apparatus,
}

/// Per-verse (and aggregable) coverage counts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Coverage {
    pub analyzed: usize,
    pub underspecified: usize,
    /// leaves the tagger chose (`:by … tagger`): never counted as analysed
    pub tagged: usize,
    pub closed: usize,
    pub ambiguous: usize,
    pub verbatim: usize,
    pub apparatus: usize,
}

impl Coverage {
    pub fn total(&self) -> usize {
        self.analyzed + self.underspecified + self.tagged + self.closed + self.ambiguous + self.verbatim + self.apparatus
    }
    pub fn add(&mut self, other: Coverage) {
        self.analyzed += other.analyzed;
        self.underspecified += other.underspecified;
        self.tagged += other.tagged;
        self.closed += other.closed;
        self.ambiguous += other.ambiguous;
        self.verbatim += other.verbatim;
        self.apparatus += other.apparatus;
    }
    pub fn count(&mut self, fate: TokenFate) {
        match fate {
            TokenFate::Analyzed => self.analyzed += 1,
            TokenFate::Underspecified => self.underspecified += 1,
            TokenFate::ClosedClass => self.closed += 1,
            TokenFate::Ambiguous => self.ambiguous += 1,
            TokenFate::Verbatim => self.verbatim += 1,
            TokenFate::Apparatus => self.apparatus += 1,
        }
    }
}

/// A titlo-written token's expansions under one row for one lexeme:
/// (row prefix, lexeme id, cells with their alternative index).
type TitloGroup<'a> = (&'a str, &'a str, &'a str, Vec<(Cell, usize)>);

/// The lifter: the lexicon, its titlo index, and the recension.
pub struct Lifter<'a> {
    pub lexicon: &'a Lexicon,
    pub titlo: TitloIndex,
    /// The closed lexemes with `pros=encl`: (letters, id), longest first.
    pub enclitics: Vec<(String, String)>,
}

/// The host of a solid enclitic as the standalone print writes it: its
/// final oxia becomes the varia (землѧ́ → землѧ̀); `None` when the host has
/// no final oxia (its accent is not on the last vowel, or it has none).
pub fn host_standalone(host: &str) -> Option<String> {
    use unicode_normalization::UnicodeNormalization;
    let chars: Vec<char> = host.nfd().collect();
    let last_oxia = chars.iter().rposition(|c| *c == '\u{301}')?;
    if chars[last_oxia + 1..].iter().any(|c| crate::orthography::is_vowel_letter(*c)) {
        return None;
    }
    let mut out = chars;
    out[last_oxia] = '\u{300}';
    Some(out.into_iter().collect::<String>().nfc().collect())
}

impl<'a> Lifter<'a> {
    pub fn new(lexicon: &'a Lexicon) -> Lifter<'a> {
        let mut enclitics: Vec<(String, String)> = lexicon
            .iter()
            .filter(|l| l.pos == crate::Pos::Closed && l.prosody() == crate::grammar::Prosody::Enclitic)
            .map(|l| (crate::orthography::strip_marks(&l.lemma), l.id.clone()))
            .collect();
        enclitics.sort_by_key(|(letters, _)| std::cmp::Reverse(letters.chars().count()));
        Lifter { lexicon, titlo: TitloIndex::build(lexicon), enclitics }
    }

    /// A token that analyses in no way as a whole may be a host with an
    /// enclitic written solid (Землѧ́же, и҆̀хже, ѻ҆́ньже): strip the enclitic,
    /// read the host as the standalone print writes it (its final oxia a
    /// varia; its jer restored), and lift `(pw host (f же.x))` when the
    /// host is one lexeme. The unit must render the token back exactly —
    /// `lift_token`'s probe checks it.
    fn lift_enclitic(&self, looked_up: &str) -> Option<(Node, usize)> {
        for (letters, id) in &self.enclitics {
            let Some(host) = looked_up.strip_suffix(letters.as_str()) else { continue };
            if host.chars().filter(|c| crate::orthography::is_vowel_letter(*c)).count() == 0 {
                continue;
            }
            let standalone = host_standalone(host);
            let candidates: Vec<String> = [standalone.clone(), Some(host.to_string()), standalone.map(|h| format!("{h}ъ")), Some(format!("{host}ъ"))].into_iter().flatten().collect();
            for candidate in candidates {
                if let Some((host, n)) = self.one_lexeme(&candidate) {
                    return Some((Node::Pw { host: Box::new(host), enclitics: vec![Node::Fn(id.clone())], apart: false }, n));
                }
            }
        }
        None
    }

    /// The one lexeme a surface reads as exactly (a closed lexeme counts
    /// as one: во́нь the contraction beside вонѧ's genitive is two), as a
    /// host node with its cell count.
    pub fn one_lexeme(&self, surface: &str) -> Option<(Node, usize)> {
        let exact: Vec<crate::Reading<'_>> = self.lexicon.readings(surface).into_iter().filter(|r| r.exact).collect();
        if exact.len() != 1 {
            return None;
        }
        Some(leaf(&exact[0].lexeme.id, &exact[0].cells))
    }

    /// The enclitic a token is, by its core: a closed enclitic (же, бо,
    /// ли) as its function word, or (3.3) a personal pronoun's clitic cell
    /// printed without its accent (мѧ, тѧ, ми, ти beside the lexicon's
    /// мѧ̀, тѧ̀, мѝ, тѝ) as the clitic leaf — with its fate.
    pub fn enclitic_node(&self, core: &str) -> Option<(Node, TokenFate)> {
        let key = crate::orthography::strip_marks(core);
        if let Some((_, id)) = self.enclitics.iter().find(|(letters, _)| *letters == key) {
            return Some((Node::Fn(id.clone()), TokenFate::ClosedClass));
        }
        if core.chars().any(|c| matches!(c, '\u{300}' | '\u{301}' | '\u{311}')) {
            return None;
        }
        // a token that is a word of its own as printed (the conjunction и҆
        // beside the clitic и҆̀) is not a clitic
        let readings = self.lexicon.readings(core);
        if readings.iter().any(|r| r.exact) {
            return None;
        }
        let clitic: Vec<crate::Reading<'_>> = readings
            .into_iter()
            .filter(|r| !r.cells.is_empty() && r.cells.iter().all(|(c, _)| matches!(c, Cell::Pron(pc) if pc.clitic)))
            .collect();
        let [r] = clitic.as_slice() else { return None };
        let (node, n) = leaf(&r.lexeme.id, &r.cells);
        Some((node, if n > 1 { TokenFate::Underspecified } else { TokenFate::Analyzed }))
    }

    /// A host whose enclitic is written apart (Землѧ́ же): the token has no
    /// exact reading because its final oxia is the unit's, the next token
    /// is an enclitic, and the host read as the standalone print (the
    /// oxia a varia) is one lexeme. Lifts `(pwa host (f же.x))` over the
    /// two tokens; the probe renders both back.
    fn lift_apart(&self, token: &str, next: &str) -> Option<(Vec<Node>, TokenFate, TokenFate)> {
        let core = token_core(token)?;
        let next_core = token_core(next)?;
        let (enclitic, enclitic_fate) = self.enclitic_node(next_core)?;
        let (looked_up, capped) = match decapitalized(core) {
            Some(low) => (low, true),
            None => (core.to_string(), false),
        };
        let has_exact = self.lexicon.readings(&looked_up).iter().any(|r| r.exact);
        let (host, n) = if enclitic_fate == TokenFate::ClosedClass {
            // же, бо, ли: a unit only where the host's accent shows it
            if has_exact {
                return None;
            }
            self.one_lexeme(&host_standalone(&looked_up)?)?
        } else if has_exact {
            // a pronoun clitic is unaccented only inside its unit: the
            // host may be printed as it stands (и҆зба́ви мѧ) …
            self.one_lexeme(&looked_up)?
        } else {
            // … or with the unit's oxia (прельсти́ мѧ)
            self.one_lexeme(&host_standalone(&looked_up)?)?
        };
        let unit = Node::Pw { host: Box::new(host), enclitics: vec![enclitic], apart: true };
        let unit = if capped { Node::Cap(Box::new(unit)) } else { unit };
        // the punctuation around the two tokens
        let lead_len = core.as_ptr() as usize - token.as_ptr() as usize;
        let (lead, rest) = token.split_at(lead_len);
        let (_, trail) = rest.split_at(core.len());
        let next_lead_len = next_core.as_ptr() as usize - next.as_ptr() as usize;
        let (next_lead, next_rest) = next.split_at(next_lead_len);
        let (_, next_trail) = next_rest.split_at(next_core.len());
        if !trail.is_empty() || !next_lead.is_empty() {
            return None;
        }
        let mut nodes: Vec<Node> = lead.chars().map(|c| Node::Punct(c.to_string())).collect();
        nodes.push(unit);
        nodes.extend(next_trail.chars().map(|c| Node::Punct(c.to_string())));
        let probe = Node::Group { head: "s".to_string(), children: nodes.clone() };
        match crate::sentence::node::render(&probe, &self.lexicon.recension) {
            Ok(rebuilt) if rebuilt == format!("{token} {next}") => Some((nodes, if n > 1 { TokenFate::Underspecified } else { TokenFate::Analyzed }, enclitic_fate)),
            _ => None,
        }
    }

    /// Lift one token's core: exactly one exact reading (one lexeme; its
    /// cells printing the token, one or several) gives a leaf, several
    /// lexemes an ambiguous verbatim leaf, none a function word or a
    /// verbatim leaf.
    pub fn lift_core(&self, core: &str) -> (Node, TokenFate) {
        let (looked_up, capped) = match decapitalized(core) {
            Some(low) => (low, true),
            None => (core.to_string(), false),
        };
        let wrap = |node: Node| if capped { Node::Cap(Box::new(node)) } else { node };
        let all: Vec<crate::Reading<'_>> = self.lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).collect();
        // the closed classes: several uninflected lexemes printing one word
        // (Polyakov lists и҆ as a conjunction and as a particle) are one
        // function word to the tree, `(f и҆)` by its surface
        let (closed_readings, exact): (Vec<_>, Vec<_>) = all.into_iter().partition(|r| r.cells.iter().all(|(c, _)| *c == Cell::Word));
        // a titlo-written token: its expansions grouped the same way
        let titlo = self.titlo.map.get(&looked_up).map(Vec::as_slice).unwrap_or(&[]);
        let mut titlo_groups: Vec<TitloGroup<'_>> = Vec::new();
        for (prefix, id, cell, alt, full) in titlo {
            // one lexeme is one reading whatever row abbreviated it (4.1:
            // ѻ҆ц҃а̀ under the rows отц and отец counted as two lexemes)
            match titlo_groups.iter_mut().find(|(_, i, _, _)| *i == id) {
                Some((_, _, _, cells)) => {
                    if !cells.iter().any(|(c, _)| c == cell) {
                        cells.push((*cell, *alt));
                    }
                }
                None => titlo_groups.push((prefix, id, full, vec![(*cell, *alt)])),
            }
        }
        let closed = crate::sentence::closed::is_closed(&looked_up) || !closed_readings.is_empty();
        let readings = exact.len() + titlo_groups.len();
        match (readings, closed) {
            (1, false) => {
                let (node, cells) = if let Some(r) = exact.first() {
                    let (node, n) = leaf(&r.lexeme.id, &r.cells);
                    (node, n)
                } else {
                    let (prefix, id, full, cells) = &titlo_groups[0];
                    let (node, n) = leaf(id, cells);
                    (Node::Abbr { prefix: (*prefix).to_string(), full: Some((*full).to_string()), child: Box::new(node) }, n)
                };
                (wrap(node), if cells > 1 { TokenFate::Underspecified } else { TokenFate::Analyzed })
            }
            (0, true) => {
                let node = match closed_readings.as_slice() {
                    // the lexeme's own variant (во beside въ) is the leaf's
                    // alternative, never a verbatim token (3.3)
                    [only] => match only.cells.iter().find(|(c, _)| *c == Cell::Word).map(|(_, a)| *a).unwrap_or(0) {
                        0 => Node::Fn(only.lexeme.id.clone()),
                        alt => Node::Lex { id: only.lexeme.id.clone(), cells: CellSet::one(Cell::Word), alt, notes: Vec::new() },
                    },
                    _ => Node::Fn(looked_up),
                };
                (wrap(node), TokenFate::ClosedClass)
            }
            (0, false) => match self.lift_enclitic(&looked_up) {
                Some((node, cells)) => (wrap(node), if cells > 1 { TokenFate::Underspecified } else { TokenFate::Analyzed }),
                None => (Node::W { surface: core.to_string(), notes: Vec::new() }, TokenFate::Verbatim),
            },
            // several lexemes (or a lexeme beside a closed-class word):
            // homonymy, recorded and never guessed
            (n, _) => (
                Node::W { surface: core.to_string(), notes: vec![("amb".to_string(), (n + usize::from(closed)).to_string())] },
                TokenFate::Ambiguous,
            ),
        }
    }

    /// Lift one token into nodes; report its fate.
    pub fn lift_token(&self, token: &str) -> (Vec<Node>, TokenFate) {
        // apparatus stays whole — the target is the verse as printed
        if is_apparatus(token) {
            return (vec![Node::W { surface: token.to_string(), notes: Vec::new() }], TokenFate::Apparatus);
        }
        let Some(core) = token_core(token) else {
            // a FREE-STANDING punctuation token (the print has e.g.
            // «а҆ссѷрі́йскъ .» in 4 Kings 17:3) — it must keep its own
            // space, so it stays a verbatim leaf, never a gluing (p …)
            return (vec![Node::W { surface: token.to_string(), notes: Vec::new() }], TokenFate::Verbatim);
        };
        let core_start = core.as_ptr() as usize - token.as_ptr() as usize;
        let (lead, rest) = token.split_at(core_start);
        let (_, trail) = rest.split_at(core.len());
        let mut nodes: Vec<Node> = lead.chars().map(|c| Node::Punct(c.to_string())).collect();
        let (core_node, fate) = self.lift_core(core);
        nodes.push(core_node);
        nodes.extend(trail.chars().map(|c| Node::Punct(c.to_string())));
        // the split must rebuild the token EXACTLY under the glue rule — the
        // print holds typographic oddities (Proverbs 15:33 opens a bracket
        // with «(,*…») that no reasonable rule should chase; when the local
        // reconstruction differs, the whole token stays verbatim. The same
        // probe is the leaf's own round-trip: a leaf that does not render
        // its token never enters a tree.
        let probe = Node::Group { head: "s".to_string(), children: nodes.clone() };
        match crate::sentence::node::render(&probe, &self.lexicon.recension) {
            Ok(rebuilt) if rebuilt == token => (nodes, fate),
            _ => (vec![Node::W { surface: token.to_string(), notes: Vec::new() }], TokenFate::Verbatim),
        }
    }

    /// Auto-lift one verse into an `(s …)` tree.
    pub fn lift_verse(&self, verse: &str) -> (Node, Coverage) {
        let mut children = Vec::new();
        let mut coverage = Coverage::default();
        let tokens = crate::sentence::node::tokenize(verse);
        let mut i = 0;
        while i < tokens.len() {
            // a host and the enclitic written apart after it are one unit
            if let Some(next) = tokens.get(i + 1)
                && let Some((nodes, fate, enclitic_fate)) = self.lift_apart(tokens[i], next)
            {
                children.extend(nodes);
                coverage.count(fate);
                coverage.count(enclitic_fate);
                i += 2;
                continue;
            }
            let (nodes, fate) = self.lift_token(tokens[i]);
            children.extend(nodes);
            coverage.count(fate);
            i += 1;
        }
        (Node::Group { head: "s".to_string(), children }, coverage)
    }
}

/// The leaf of one lexeme's reading: every cell that prints the token,
/// the alternative index of the first; the count of cells.
pub fn leaf(id: &str, cells: &[(Cell, usize)]) -> (Node, usize) {
    if cells.iter().all(|(c, _)| *c == Cell::Word) {
        return (Node::Fn(id.to_string()), 1);
    }
    let mut sorted: Vec<(Cell, usize)> = cells.to_vec();
    sorted.sort();
    let set = CellSet::new(sorted.iter().map(|(c, _)| *c).collect()).expect("one part of speech");
    let alt = sorted.iter().find(|(c, _)| *c == set.first()).map(|(_, a)| *a).unwrap_or(0);
    let n = set.len();
    (Node::Lex { id: id.to_string(), cells: set, alt, notes: Vec::new() }, n)
}

/// Characters that split off a token's edges as `(p …)` nodes.
fn is_punct(c: char) -> bool {
    // ꙳ (U+A673): the service books' footnote mark, glued to the word before it
    matches!(c, '.' | ',' | ':' | ';' | '!' | '?' | '(' | ')' | '«' | '»' | '꙳')
}

/// The apparatus of the pinned print: the bracketed notes with their
/// mark (꙾, [), the bare asterisk and the asterisk with the arrow (`*`,
/// `*↑`: 117 tokens), and the notes' language labels є҆вр 37, гре́ч 20
/// (3.3 Part 2). None of these is a word of the text.
pub fn is_apparatus(token: &str) -> bool {
    token.contains('꙾')
        || token.contains('[')
        || (!token.is_empty() && token.chars().all(|c| matches!(c, '*' | '↑')))
        || matches!(token, "є\u{486}вр" | "гре\u{301}ч")
        || is_numeral(token)
        // the service books' marks (4.1): заⷱ҇ (зача́ло, the pericope), сⷯ
        // (сті́хъ, the verse), a page reference («225>>»), a bare number
        || matches!(token.trim_matches(|c: char| is_punct(c)), "за\u{2df1}\u{487}" | "с\u{2def}")
        || token.contains('>')
        || (!token.is_empty() && token.chars().all(|c| c.is_ascii_digit()))
}

/// A Cyrillic numeral under a titlo (к҃а, д҃і, ҂а҃, the ordinal suffix
/// д҃-ѧ): the print's chapter, verse and page numbers — a mark, not a
/// word (4.1: the service books number everything).
pub fn is_numeral(token: &str) -> bool {
    let core = token.trim_matches(|c: char| c.is_ascii_punctuation() && c != '-');
    let Some((digits, _suffix)) = core.split_once('-').or(Some((core, ""))) else { return false };
    if !digits.contains('\u{483}') {
        return false;
    }
    let letters: Vec<char> = digits.chars().filter(|c| !matches!(c, '\u{483}' | '\u{2de0}'..='\u{2dff}' | '\u{482}')).collect();
    !letters.is_empty() && letters.len() <= 4 && letters.iter().all(|c| matches!(c, 'а' | 'в' | 'г' | 'д' | 'є' | 'е' | 'ѕ' | 'з' | 'и' | 'ѳ' | 'і' | 'к' | 'л' | 'м' | 'н' | 'ѯ' | 'ѻ' | 'о' | 'п' | 'ч' | 'р' | 'с' | 'т' | 'ѵ' | 'ф' | 'х' | 'ѱ' | 'ѡ' | 'ц' | '҂'))
}

/// The word inside a verse token: leading and trailing punctuation
/// removed; `None` for an apparatus token or bare punctuation.
pub fn token_core(token: &str) -> Option<&str> {
    if is_apparatus(token) {
        return None;
    }
    let core_start = token.len() - token.trim_start_matches(is_punct).len();
    let core_end = token.trim_end_matches(is_punct).len();
    (core_start < core_end).then(|| &token[core_start..core_end])
}

pub fn decapitalized(word: &str) -> Option<String> {
    let first = word.chars().next()?;
    if !first.is_uppercase() {
        return None;
    }
    Some(first.to_lowercase().chain(word.chars().skip(1)).collect())
}

/// The recension a lifter works in (the Bible is Synodal).
pub const RECENSION: Recension = Recension::Synodal;
