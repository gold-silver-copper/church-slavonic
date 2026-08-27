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
//! Two tables are reported, per part of speech per recension — the README's
//! two tables:
//! - recall through any key (the headline), with the variant gap: attested
//!   forms no published key produces;
//! - bare-lemma correctness ([`BareScore`]): does the natural bare-lemma call
//!   return the PRIMARY (first-listed) attested form? The recall metric unions
//!   every key, so it is blind to a standard form demoted to an `_n` key.
//!
//! Run `cargo xtask accuracy` before and after any rule or policy change so the
//! change carries a number, not an anecdote. Misses are written per POS and
//! recension to `data/intermediate/*_misses.tsv`.

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

pub fn run_checks(intermediate_dir: &Path, artifacts_dir: &Path) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "checks")]
    {
        harness::run(intermediate_dir, artifacts_dir)
    }
    #[cfg(not(feature = "checks"))]
    {
        let _ = (intermediate_dir, artifacts_dir);
        Err("extractor was built without the `checks` feature; run `cargo xtask accuracy`.".into())
    }
}

#[cfg(feature = "checks")]
mod harness {
    use super::{BareScore, Score, score_bare, score_slot};
    use crate::assign::split_key;
    use crate::bootstrap::parse_phf_pairs;
    use crate::cells::{
        CASES, GENDERS, NUMBERS, PERSONS, PRONOUN_KEY, Pos, VERB_BLOCKS, recension_of_tag,
        rule_matches,
    };
    use crate::extract::{Lexemes, gather};
    use church_slavonic::ChurchSlavonic;
    use church_slavonic_core::grammar::*;
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
                ChurchSlavonic::pronoun(&person, number, gender, case, r).to_string()
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

    fn label(tag: &str) -> &'static str {
        match tag {
            "ocs" => "OCS",
            _ => "Synodal",
        }
    }

    fn pos_label(pos: Pos) -> &'static str {
        match pos {
            Pos::Noun => "Nouns",
            Pos::Adj => "Adjectives",
            Pos::Verb => "Verbs",
            Pos::Pronoun => "Pronouns",
        }
    }

    pub fn run(intermediate_dir: &Path, artifacts_dir: &Path) -> Result<(), Box<dyn Error>> {
        let lexemes: Lexemes = gather(intermediate_dir)?;
        let published = published_keys()?;
        let mut reports: BTreeMap<(Pos, &'static str), Report> = BTreeMap::new();
        for (key, observations) in &lexemes {
            let Some(recension) = recension_of_tag(key.tag) else {
                continue;
            };
            let same = |a: &str, b: &str| rule_matches(&recension, a, b);
            let keys = if key.pos == Pos::Pronoun {
                vec![PRONOUN_KEY.to_string()]
            } else {
                keys_for(&published, key.tag, key.pos, &key.lemma)
            };
            let report = reports.entry((key.pos, key.tag)).or_default();
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

        let mut out = String::new();
        out.push_str("\nRecall through any published key (attested source slots the library reproduces via the bare lemma or any `_n` key):\n\n");
        out.push_str("| Part of Speech | Recension | Correct / Total | Accuracy | Variant gap |\n");
        out.push_str("|----------------|-----------|-----------------|----------|-------------|\n");
        for ((pos, tag), r) in &reports {
            let _ = writeln!(
                out,
                "| **{}** | {} | {} / {} | {} | {} |",
                pos_label(*pos),
                label(tag),
                r.recall.matched_slots,
                r.recall.total_slots,
                r.recall.percent_display(),
                r.recall.unreachable_forms
            );
        }
        out.push_str("\nBare-lemma correctness (does the natural bare-lemma call return the primary, first-listed, attested form?):\n\n");
        out.push_str("| Part of Speech | Recension | Bare Primary / Total | Bare Accuracy | Demoted to `_n` |\n");
        out.push_str("|----------------|-----------|----------------------|---------------|-----------------|\n");
        for ((pos, tag), r) in &reports {
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
                label(tag),
                r.bare.bare_primary_hits,
                r.bare.total,
                pct,
                r.bare.demoted
            );
        }
        print!("{out}");
        for ((pos, tag), r) in &reports {
            let path = artifacts_dir.join(format!("{}_{}_misses.tsv", pos.label(), tag));
            fs::write(
                &path,
                format!("lemma\tcell\tattested\tproduced\tkind\n{}", r.misses),
            )?;
        }
        println!(
            "\nMisses written to {}/<pos>_<recension>_misses.tsv",
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
