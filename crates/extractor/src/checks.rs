//! Source-driven accuracy measurement (feature `checks`, which links the
//! `church-slavonic` crate so the CURRENT committed tables + rule engine are
//! what gets measured).
//!
//! Scoring is per SLOT, not per attested form: a source can list several
//! valid variants for one cell (`сꙑнови` and `сꙑноу`), and a per-form metric
//! could never reach 100% no matter which variant the library picks. A slot
//! counts as a hit when the library's output (through any sense-numbered key
//! of the lemma) matches one of the attested forms under the source's
//! comparison policy ([`crate::cells::rule_matches`]: accent-blind for the
//! Kaikki dump, exact for the accented Alypy print).
//!
//! Two tables are reported, per part of speech per source (each source scored
//! on its own against the tables all of them fed) — the README's two tables:
//! - recall through any key (the headline), with the variant gap: attested
//!   forms no published key produces;
//! - bare-lemma correctness ([`BareScore`]): does the natural bare-lemma call
//!   return the PRIMARY (first-listed) attested form? The recall metric unions
//!   every key, so it is blind to a standard form demoted to an `_n` key.
//!
//! The two Old Church Slavonic treebanks ([`crate::treebank`]) are scored the
//! same way — every annotated token whose features name a schema cell is a
//! slot, the lemma and the surface compared through
//! [`church_slavonic_core::orthography::comparison_key`] (manuscript spelling
//! varies) — and reported as "corpus recall" rows; they feed no table.
//!
//! Run `cargo xtask accuracy` before and after any rule or policy change so the
//! change carries a number, not an anecdote. Misses are written per POS and
//! recension to `data/intermediate/*_misses.tsv`, and per treebank to
//! `data/intermediate/<treebank>_misses.tsv`.

use std::error::Error;
use std::path::Path;

/// One score: slots reproduced, plus the variant gap.
#[derive(Debug, Clone, Copy, Default)]
pub struct Score {
    pub matched_slots: u64,
    pub total_slots: u64,
    /// Attested forms no published key produces for their slot.
    pub unreachable_forms: u64,
}

impl Score {
    pub fn percent(&self) -> f64 {
        if self.total_slots == 0 {
            100.0
        } else {
            100.0 * self.matched_slots as f64 / self.total_slots as f64
        }
    }

    /// The percentage as a display string that never rounds a NON-perfect score
    /// up to `100.00%`.
    pub fn percent_display(&self) -> String {
        if self.total_slots == 0 || self.matched_slots >= self.total_slots {
            return "100.00%".to_string();
        }
        let pct = self.percent();
        for prec in 2..=6usize {
            let factor = 10f64.powi(prec as i32);
            if (pct * factor).round() / factor < 100.0 {
                return format!("{pct:.prec$}%");
            }
        }
        format!("{:.2}%", (pct * 100.0).floor() / 100.0)
    }
}

/// Bare-lemma correctness — what the NATURAL call returns, not what is merely
/// reachable.
#[derive(Debug, Clone, Copy, Default)]
pub struct BareScore {
    pub bare_primary_hits: u64,
    pub total: u64,
    /// Slots the recall metric hits, yet the bare lemma does not return the
    /// primary although some other key produces it — a demotion to `_n`.
    pub demoted: u64,
}

/// Score one slot: the attested set against the outputs across keys.
#[cfg(feature = "checks")]
fn score_slot(
    score: &mut Score,
    attested: &[String],
    produced: &[String],
    same: impl Fn(&str, &str) -> bool,
) -> bool {
    score.total_slots += 1;
    let hit = attested.iter().any(|a| produced.iter().any(|p| same(a, p)));
    if hit {
        score.matched_slots += 1;
    }
    score.unreachable_forms += attested
        .iter()
        .filter(|a| !produced.iter().any(|p| same(a, p)))
        .count() as u64;
    hit
}

#[cfg(feature = "checks")]
fn score_bare(
    bs: &mut BareScore,
    bare: &str,
    primary: &str,
    produced: &[String],
    same: impl Fn(&str, &str) -> bool,
) -> bool {
    bs.total += 1;
    let is_primary = same(primary, bare);
    if is_primary {
        bs.bare_primary_hits += 1;
    } else if produced.iter().any(|p| same(primary, p)) {
        bs.demoted += 1;
    }
    is_primary
}

pub fn run_checks(
    intermediate_dir: &Path,
    artifacts_dir: &Path,
    sources_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "checks")]
    {
        harness::run(intermediate_dir, artifacts_dir, sources_dir)
    }
    #[cfg(not(feature = "checks"))]
    {
        let _ = (intermediate_dir, artifacts_dir, sources_dir);
        Err("extractor was built without the `checks` feature; run `cargo xtask accuracy`.".into())
    }
}

#[cfg(feature = "checks")]
mod harness {
    use super::{BareScore, Score, score_bare, score_slot};
    use crate::assign::split_key;
    use crate::bootstrap::parse_phf_pairs;
    use crate::cells::{
        CASES, GENDERS, NUMBERS, PERSONS, Pos, VERB_BLOCKS, recension_of_tag, rule_matches,
    };
    use crate::extract::{Lexemes, Source, gather_sources};
    use crate::treebank::{Corpus, load_syntacticus, load_ud_proiel};
    use church_slavonic::ChurchSlavonic;
    use church_slavonic_core::grammar::*;
    use church_slavonic_core::orthography::comparison_key;
    use std::collections::BTreeMap;
    use std::error::Error;
    use std::fmt::Write as _;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn generated_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../church-slavonic/generated")
    }

    /// Published keys per `(tag, pos, lemma)`.
    type Published = BTreeMap<(String, Pos, String), Vec<String>>;

    /// Published keys per `(tag, pos, lemma)`, read from the ACTUAL generated
    /// tables — never a fixed probe window.
    fn published_keys() -> Result<Published, Box<dyn Error>> {
        let mut out = Published::new();
        for pos in Pos::ALL {
            for (key, _) in parse_phf_pairs(generated_dir().join(pos.file_name()))? {
                let Some((tag, rest)) = key.split_once(':') else {
                    continue;
                };
                let lemma = split_key(rest).map(|(b, _)| b).unwrap_or(rest);
                out.entry((tag.to_string(), pos, lemma.to_string()))
                    .or_default()
                    .push(rest.to_string());
            }
        }
        Ok(out)
    }

    /// The keys to query for one lemma: the bare lemma (the rule fallback is
    /// always reachable) plus every suffixed key the tables publish.
    fn keys_for(published: &Published, tag: &str, pos: Pos, lemma: &str) -> Vec<String> {
        let mut keys = vec![lemma.to_string()];
        if let Some(list) = published.get(&(tag.to_string(), pos, lemma.to_string())) {
            keys.extend(list.iter().filter(|k| *k != lemma).cloned());
        }
        keys
    }

    /// The library's answer for cell `i` of `key`'s row.
    fn produce(pos: Pos, key: &str, i: usize, r: &Recension) -> String {
        match pos {
            Pos::Noun => ChurchSlavonic::noun(key, &CASES[i % 7], &NUMBERS[i / 7], r),
            Pos::Adj => {
                let case = &CASES[i % 7];
                let rest = i / 7;
                let number = &NUMBERS[rest % 3];
                let gender = &GENDERS[(rest / 3) % 3];
                let degree = if rest / 9 == 0 {
                    Degree::Positive
                } else {
                    Degree::Comparative
                };
                ChurchSlavonic::adj(key, case, number, gender, &degree, r)
            }
            Pos::Verb => {
                let (person, number, tense, form) = if i < 36 {
                    let (tense, form) = VERB_BLOCKS[i / 9];
                    (PERSONS[i % 3], NUMBERS[(i % 9) / 3], tense, form)
                } else {
                    let tense = if i == 36 {
                        Tense::Present
                    } else {
                        Tense::Aorist
                    };
                    (Person::Third, Number::Singular, tense, Form::Participle)
                };
                ChurchSlavonic::verb(key, &person, &number, &tense, &form, r)
            }
            Pos::Pronoun => {
                let (person, rest) = if i < 18 {
                    (Person::First, i)
                } else if i < 36 {
                    (Person::Second, i - 18)
                } else {
                    (Person::Third, i - 36)
                };
                let case = &CASES[rest % 6];
                let number = &NUMBERS[(rest / 6) % 3];
                let gender = &GENDERS[(rest / 6) / 3];
                ChurchSlavonic::pronoun_sense(key, &person, number, gender, case, r).to_string()
            }
        }
    }

    /// Every attested form of a slot across observations, primary first.
    fn slot_forms(observations: &[crate::extract::Observation], i: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for obs in observations {
            for f in &obs.cells[i] {
                if !out.contains(f) {
                    out.push(f.clone());
                }
            }
        }
        out
    }

    #[derive(Default)]
    struct Report {
        recall: Score,
        bare: BareScore,
        misses: String,
    }

    fn pos_label(pos: Pos) -> &'static str {
        match pos {
            Pos::Noun => "Nouns",
            Pos::Adj => "Adjectives",
            Pos::Verb => "Verbs",
            Pos::Pronoun => "Pronouns",
        }
    }

    /// Each source is scored on its own against the published tables (which
    /// every source fed): the number measures the table/rule machinery, not
    /// generalisation. The treebanks fed nothing: their rows measure it.
    pub fn run(
        intermediate_dir: &Path,
        artifacts_dir: &Path,
        sources_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let published = published_keys()?;
        let mut reports: BTreeMap<(Pos, Source), Report> = BTreeMap::new();
        for source in Source::ALL {
            let lexemes: Lexemes = gather_sources(intermediate_dir, &[source])?;
            score_source(source, &lexemes, &published, &mut reports);
        }
        let corpora: Vec<CorpusReport> = [
            load_ud_proiel(sources_dir, artifacts_dir)?,
            load_syntacticus(sources_dir, artifacts_dir)?,
        ]
        .into_iter()
        .flatten()
        .map(|corpus| score_corpus(corpus, &published))
        .collect();
        report(&reports, &corpora, artifacts_dir)
    }

    /// A scored treebank: per part of speech, the slots reproduced.
    struct CorpusReport {
        corpus: Corpus,
        scores: BTreeMap<Pos, Score>,
        misses: String,
    }

    /// Score a treebank: a slot is a hit when some published key of a lemma
    /// spelled like the token's (its [`comparison_key`]) — or the bare
    /// lemma through the rule — produces the surface up to spelling.
    fn score_corpus(corpus: Corpus, published: &Published) -> CorpusReport {
        let ocs = Recension::OldChurchSlavonic;
        let mut by_spelling: BTreeMap<(Pos, String), Vec<String>> = BTreeMap::new();
        for ((tag, pos, lemma), keys) in published {
            if tag == "ocs" {
                by_spelling
                    .entry((*pos, comparison_key(lemma)))
                    .or_default()
                    .extend(keys.iter().cloned());
            }
        }
        let mut scores: BTreeMap<Pos, Score> = BTreeMap::new();
        let mut misses = String::new();
        for slot in &corpus.slots {
            let mut keys = vec![slot.lemma.clone()];
            if let Some(list) = by_spelling.get(&(slot.pos, comparison_key(&slot.lemma))) {
                keys.extend(list.iter().filter(|k| **k != slot.lemma).cloned());
            }
            let produced: Vec<String> = keys
                .iter()
                .map(|k| produce(slot.pos, k, slot.cell, &ocs))
                .collect();
            let hit = score_slot(
                scores.entry(slot.pos).or_default(),
                std::slice::from_ref(&slot.surface),
                &produced,
                |a, b| comparison_key(a) == comparison_key(b),
            );
            if !hit {
                let _ = writeln!(
                    misses,
                    "{}\t{}\t{}\t{}\t{}",
                    slot.pos.label(),
                    slot.lemma,
                    slot.cell,
                    slot.surface,
                    produced.join(", ")
                );
            }
        }
        CorpusReport {
            corpus,
            scores,
            misses,
        }
    }

    fn score_source(
        source: Source,
        lexemes: &Lexemes,
        published: &Published,
        reports: &mut BTreeMap<(Pos, Source), Report>,
    ) {
        for (key, observations) in lexemes {
            let Some(recension) = recension_of_tag(key.tag) else {
                continue;
            };
            let same = |a: &str, b: &str| rule_matches(&recension, a, b);
            let keys = keys_for(published, key.tag, key.pos, &key.lemma);
            let report = reports.entry((key.pos, source)).or_default();
            for i in 0..key.pos.arity() {
                let attested = slot_forms(observations, i);
                if attested.is_empty() {
                    continue;
                }
                let produced: Vec<String> = keys
                    .iter()
                    .map(|k| produce(key.pos, k, i, &recension))
                    .collect();
                let hit = score_slot(&mut report.recall, &attested, &produced, same);
                let bare_hit = score_bare(
                    &mut report.bare,
                    &produced[0],
                    &attested[0],
                    &produced,
                    same,
                );
                if !hit || !bare_hit {
                    let _ = writeln!(
                        report.misses,
                        "{}\t{}\t{}\t{}\t{}",
                        key.lemma,
                        i,
                        attested.join(", "),
                        produced.join(", "),
                        if hit {
                            "demoted-or-bare-miss"
                        } else {
                            "unreachable"
                        }
                    );
                }
            }
        }
    }

    fn report(
        reports: &BTreeMap<(Pos, Source), Report>,
        corpora: &[CorpusReport],
        artifacts_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let mut out = String::new();
        out.push_str("\nRecall through any published key (attested source slots the library reproduces via the bare lemma or any `_n` key):\n\n");
        out.push_str("| Part of Speech | Recension | Correct / Total | Accuracy | Variant gap |\n");
        out.push_str("|----------------|-----------|-----------------|----------|-------------|\n");
        for ((pos, source), r) in reports {
            let _ = writeln!(
                out,
                "| **{}** | {} | {} / {} | {} | {} |",
                pos_label(*pos),
                source.recension_label(),
                r.recall.matched_slots,
                r.recall.total_slots,
                r.recall.percent_display(),
                r.recall.unreachable_forms
            );
        }
        for c in corpora {
            for (pos, score) in &c.scores {
                let _ = writeln!(
                    out,
                    "| **{}** | {} | {} / {} | {} | {} |",
                    pos_label(*pos),
                    c.corpus.label,
                    score.matched_slots,
                    score.total_slots,
                    score.percent_display(),
                    score.unreachable_forms
                );
            }
        }
        for c in corpora {
            let _ = writeln!(
                out,
                "\n{}: {} tokens, {} slots mapped, {} skipped:{}",
                c.corpus.label,
                c.corpus.tokens,
                c.corpus.slots.len(),
                c.corpus.skipped_total(),
                c.corpus
                    .skipped
                    .iter()
                    .map(|(reason, n)| format!(" {reason}={n};"))
                    .collect::<String>()
            );
        }
        out.push_str("\nBare-lemma correctness (does the natural bare-lemma call return the primary, first-listed, attested form?):\n\n");
        out.push_str("| Part of Speech | Recension | Bare Primary / Total | Bare Accuracy | Demoted to `_n` |\n");
        out.push_str("|----------------|-----------|----------------------|---------------|-----------------|\n");
        for ((pos, source), r) in reports {
            let pct = Score {
                matched_slots: r.bare.bare_primary_hits,
                total_slots: r.bare.total,
                unreachable_forms: 0,
            }
            .percent_display();
            let _ = writeln!(
                out,
                "| **{}** | {} | {} / {} | {} | {} |",
                pos_label(*pos),
                source.recension_label(),
                r.bare.bare_primary_hits,
                r.bare.total,
                pct,
                r.bare.demoted
            );
        }
        print!("{out}");
        for ((pos, source), r) in reports {
            let path = artifacts_dir.join(format!("{}_{}_misses.tsv", pos.label(), source.label()));
            fs::write(
                &path,
                format!("lemma\tcell\tattested\tproduced\tkind\n{}", r.misses),
            )?;
        }
        for c in corpora {
            let path = artifacts_dir.join(format!("{}_misses.tsv", c.corpus.file_label));
            fs::write(
                &path,
                format!("pos\tlemma\tcell\tsurface\tproduced\n{}", c.misses),
            )?;
        }
        println!(
            "\nMisses written to {}/<pos>_<source>_misses.tsv",
            artifacts_dir.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_display_never_rounds_an_imperfect_score_up_to_100() {
        let s = Score {
            matched_slots: 121548,
            total_slots: 121550,
            unreachable_forms: 0,
        };
        assert_eq!(s.percent_display(), "99.998%");
        let s = Score {
            matched_slots: 3,
            total_slots: 3,
            unreachable_forms: 0,
        };
        assert_eq!(s.percent_display(), "100.00%");
        assert_eq!(Score::default().percent_display(), "100.00%");
    }

    #[cfg(feature = "checks")]
    #[test]
    fn slot_and_bare_scoring() {
        let same = |a: &str, b: &str| a == b;
        let mut s = Score::default();
        let att = ["а".to_string(), "б".to_string()];
        let prod = ["б".to_string(), "в".to_string()];
        assert!(score_slot(&mut s, &att, &prod, same));
        assert_eq!(
            (s.matched_slots, s.total_slots, s.unreachable_forms),
            (1, 1, 1)
        );
        let mut b = BareScore::default();
        assert!(!score_bare(&mut b, "в", "б", &prod, same));
        assert_eq!((b.bare_primary_hits, b.total, b.demoted), (0, 1, 1));
    }
}
