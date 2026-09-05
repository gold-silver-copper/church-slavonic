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

use crate::treebank::node::Node;
use church_slavonic::cell::{Cell, CellSet};
use church_slavonic::{Lexicon, Recension};
use std::collections::HashMap;

/// The titlo index: abbreviated surface → (row prefix, id, cell, alt).
pub struct TitloIndex {
    map: HashMap<String, Vec<(String, String, Cell, usize)>>,
}

impl TitloIndex {
    pub fn build(lexicon: &Lexicon) -> TitloIndex {
        let mut map: HashMap<String, Vec<(String, String, Cell, usize)>> = HashMap::new();
        for row in crate::treebank::titlo::rows() {
            // the row's lemma: every lexeme whose lemma prints as it
            let key = church_slavonic::orthography::comparison_key(row.lemma);
            for lexeme in lexicon.iter().filter(|l| church_slavonic::orthography::comparison_key(&l.lemma) == key) {
                for cell in lexeme.cells() {
                    for (alt, form) in lexeme.forms(cell).into_iter().enumerate() {
                        let full = form.print(lexicon.recension);
                        if let Some(abbreviated) = crate::treebank::titlo::abbreviate(&full, row) {
                            let entry = map.entry(abbreviated).or_default();
                            let item = (row.abbr.to_string(), lexeme.id.clone(), cell, alt);
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

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// The cells of one lexeme that abbreviate to `surface` under the
    /// titlo row `prefix` (the abbreviation erases the accent that tells
    /// дꙋ́хъ from дꙋ̑хъ, so дх҃ъ is nom.sg|gen.pl|acc.pl).
    pub fn cells(&self, surface: &str, prefix: &str, id: &str) -> Option<CellSet> {
        let cells: Vec<Cell> = self.map.get(surface)?.iter().filter(|(p, i, _, _)| p == prefix && i == id).map(|(_, _, c, _)| *c).collect();
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
type TitloGroup<'a> = (&'a str, &'a str, Vec<(Cell, usize)>);

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
    if chars[last_oxia + 1..].iter().any(|c| church_slavonic::orthography::is_vowel_letter(*c)) {
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
            .filter(|l| l.pos == church_slavonic::Pos::Closed && l.prosody() == church_slavonic::grammar::Prosody::Enclitic)
            .map(|l| (church_slavonic::orthography::strip_marks(&l.lemma), l.id.clone()))
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
            if host.chars().filter(|c| church_slavonic::orthography::is_vowel_letter(*c)).count() == 0 {
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
    fn one_lexeme(&self, surface: &str) -> Option<(Node, usize)> {
        let exact: Vec<church_slavonic::Reading<'_>> = self.lexicon.readings(surface).into_iter().filter(|r| r.exact).collect();
        if exact.len() != 1 {
            return None;
        }
        Some(leaf(&exact[0].lexeme.id, &exact[0].cells))
    }

    /// The enclitic a token is, by its core (же, бо, ли): its id.
    fn enclitic_id(&self, core: &str) -> Option<&str> {
        let key = church_slavonic::orthography::strip_marks(core);
        self.enclitics.iter().find(|(letters, _)| *letters == key).map(|(_, id)| id.as_str())
    }

    /// A host whose enclitic is written apart (Землѧ́ же): the token has no
    /// exact reading because its final oxia is the unit's, the next token
    /// is an enclitic, and the host read as the standalone print (the
    /// oxia a varia) is one lexeme. Lifts `(pwa host (f же.x))` over the
    /// two tokens; the probe renders both back.
    fn lift_apart(&self, token: &str, next: &str) -> Option<(Vec<Node>, TokenFate)> {
        let core = token_core(token)?;
        let next_core = token_core(next)?;
        let enclitic = self.enclitic_id(next_core)?.to_string();
        let (looked_up, capped) = match decapitalized(core) {
            Some(low) => (low, true),
            None => (core.to_string(), false),
        };
        if self.lexicon.readings(&looked_up).iter().any(|r| r.exact) {
            return None;
        }
        let standalone = host_standalone(&looked_up)?;
        let (host, n) = self.one_lexeme(&standalone)?;
        let unit = Node::Pw { host: Box::new(host), enclitics: vec![Node::Fn(enclitic)], apart: true };
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
        match crate::treebank::node::render(&probe, &self.lexicon.recension) {
            Ok(rebuilt) if rebuilt == format!("{token} {next}") => Some((nodes, if n > 1 { TokenFate::Underspecified } else { TokenFate::Analyzed })),
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
        let all: Vec<church_slavonic::Reading<'_>> = self.lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).collect();
        // the closed classes: several uninflected lexemes printing one word
        // (Polyakov lists и҆ as a conjunction and as a particle) are one
        // function word to the tree, `(f и҆)` by its surface
        let (closed_readings, exact): (Vec<_>, Vec<_>) = all.into_iter().partition(|r| r.cells.iter().all(|(c, _)| *c == Cell::Word));
        // a titlo-written token: its expansions grouped the same way
        let titlo = self.titlo.map.get(&looked_up).map(Vec::as_slice).unwrap_or(&[]);
        let mut titlo_groups: Vec<TitloGroup<'_>> = Vec::new();
        for (prefix, id, cell, alt) in titlo {
            match titlo_groups.iter_mut().find(|(p, i, _)| *p == prefix && *i == id) {
                Some((_, _, cells)) => {
                    if !cells.iter().any(|(c, _)| c == cell) {
                        cells.push((*cell, *alt));
                    }
                }
                None => titlo_groups.push((prefix, id, vec![(*cell, *alt)])),
            }
        }
        let closed = crate::treebank::closed::is_closed(&looked_up) || !closed_readings.is_empty();
        let readings = exact.len() + titlo_groups.len();
        match (readings, closed) {
            (1, false) => {
                let (node, cells) = if let Some(r) = exact.first() {
                    let (node, n) = leaf(&r.lexeme.id, &r.cells);
                    (node, n)
                } else {
                    let (prefix, id, cells) = &titlo_groups[0];
                    let (node, n) = leaf(id, cells);
                    (Node::Abbr { prefix: (*prefix).to_string(), child: Box::new(node) }, n)
                };
                (wrap(node), if cells > 1 { TokenFate::Underspecified } else { TokenFate::Analyzed })
            }
            (0, true) => {
                let node = match closed_readings.as_slice() {
                    [only] => Node::Fn(only.lexeme.id.clone()),
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
        if token.contains('꙾') || token.contains('[') {
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
        match crate::treebank::node::render(&probe, &self.lexicon.recension) {
            Ok(rebuilt) if rebuilt == token => (nodes, fate),
            _ => (vec![Node::W { surface: token.to_string(), notes: Vec::new() }], TokenFate::Verbatim),
        }
    }

    /// Auto-lift one verse into an `(s …)` tree.
    pub fn lift_verse(&self, verse: &str) -> (Node, Coverage) {
        let mut children = Vec::new();
        let mut coverage = Coverage::default();
        let tokens = crate::treebank::node::tokenize(verse);
        let mut i = 0;
        while i < tokens.len() {
            // a host and the enclitic written apart after it are one unit
            if let Some(next) = tokens.get(i + 1)
                && let Some((nodes, fate)) = self.lift_apart(tokens[i], next)
            {
                children.extend(nodes);
                coverage.count(fate);
                coverage.count(TokenFate::ClosedClass);
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
fn leaf(id: &str, cells: &[(Cell, usize)]) -> (Node, usize) {
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
    matches!(c, '.' | ',' | ':' | ';' | '!' | '?' | '(' | ')' | '«' | '»')
}

/// The word inside a verse token: leading and trailing punctuation
/// removed; `None` for an apparatus token or bare punctuation.
pub fn token_core(token: &str) -> Option<&str> {
    if token.contains('꙾') || token.contains('[') {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treebank::node::render;
    use std::sync::OnceLock;

    fn lifter() -> &'static Lifter<'static> {
        static L: OnceLock<Lifter<'static>> = OnceLock::new();
        L.get_or_init(|| Lifter::new(Lexicon::synodal()))
    }

    #[test]
    fn lifting_preserves_the_round_trip() {
        let verse = "Въ нача́лѣ сотворѝ бг҃ъ не́бо и҆ зе́млю.";
        let (tree, coverage) = lifter().lift_verse(verse);
        assert_eq!(render(&tree, &RECENSION).unwrap(), verse);
        assert!(coverage.analyzed >= 2, "{coverage:?}");
        let text = crate::treebank::sexpr::print(&crate::treebank::node::to_sexpr(&tree));
        assert!(text.contains("(abbr \"бг҃\" (n богъ.n :case nom :num sg))") || text.contains(":amb"), "{text}");
    }

    #[test]
    fn the_pitfall_verse_lifts_without_touching_the_apparatus() {
        let verse = "и҆ речѐ ю҆нѣ́йшїй ꙾є҆ю̀꙾[26] ѻ҆тцꙋ̀: ѻ҆́тче, да́ждь мѝ досто́йнꙋю ча́сть и҆мѣ́нїѧ.";
        let (tree, coverage) = lifter().lift_verse(verse);
        assert_eq!(render(&tree, &RECENSION).unwrap(), verse);
        assert_eq!(coverage.apparatus, 1);
    }

    #[test]
    fn syncretism_is_an_underspecified_leaf() {
        // свѣ́тъ: one lexeme, the cells its paradigm does not tell apart
        let (node, fate) = lifter().lift_core("свѣ́тъ");
        assert_eq!(fate, TokenFate::Underspecified);
        let Node::Lex { id, cells, alt, .. } = node else { panic!("{node:?}") };
        assert_eq!(id, "свѣтъ.n");
        assert_eq!(cells.name(), "nom|acc.sg");
        assert_eq!(alt, 0);
        // the leaf writes the set as a disjunctive feature
        let text = crate::treebank::sexpr::print(&crate::treebank::node::to_sexpr(&Node::Lex { id, cells, alt, notes: Vec::new() }));
        assert_eq!(text, "(n свѣтъ.n :case nom|acc :num sg)");
    }

    #[test]
    fn homonymy_is_recorded_never_guessed() {
        // дꙋ́хъ: the noun's nominative and дꙋти's aorist — two lexemes
        let (node, fate) = lifter().lift_core("дꙋ́хъ");
        assert_eq!(fate, TokenFate::Ambiguous);
        assert!(matches!(node, Node::W { ref notes, .. } if notes.iter().any(|(k, v)| k == "amb" && v == "2")), "{node:?}");
    }

    #[test]
    fn a_titlo_token_names_every_cell_the_abbreviation_hides() {
        // дх҃ъ: the accent that tells дꙋ́хъ (nom.sg) from дꙋ̑хъ (gen.pl,
        // acc.pl) is gone under the titlo
        let (node, fate) = lifter().lift_core("дх҃ъ");
        assert_eq!(fate, TokenFate::Underspecified);
        let Node::Abbr { prefix, child } = node else { panic!("{node:?}") };
        assert_eq!(prefix, "дх҃");
        let Node::Lex { cells, .. } = *child else { panic!("{child:?}") };
        assert_eq!(cells.name(), "nom.sg|gen.pl|acc.pl");
        assert_eq!(lifter().titlo.cells("дх҃ъ", "дх҃", "дꙋхъ.n").map(|c| c.name()), Some("nom.sg|gen.pl|acc.pl".to_string()));
    }
}
