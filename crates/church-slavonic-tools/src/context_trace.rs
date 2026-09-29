//! Replay-checked export of legacy local rule proposals and raw source readings.
use crate::{
    analysis_document,
    treebank::{node, sexpr},
};
use church_slavonic::{
    Lexicon,
    matching::MatchPolicy,
    sentence::{Node, Sentence},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, io::Write};

const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub rule: String,
    pub applied_by_default: bool,
    pub child: usize,
    pub context: Vec<String>,
    pub proposed: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub version: u32,
    pub interpretation: String,
    pub limitations: String,
    pub source_analysis: analysis_document::Record,
    pub input: String,
    pub proposed: String,
    pub events: Vec<Event>,
}
fn tree(node: &Node) -> String {
    sexpr::print(&node::to_sexpr(node))
}

pub fn evaluate(lexicon: &Lexicon, text: &str) -> Result<Record, Box<dyn Error>> {
    if text.len() > 4096 {
        return Err("context trace source exceeds 4096 bytes".into());
    }
    let sentence = Sentence::parse(lexicon, text);
    let trace = sentence.contextual_trace()?;
    Ok(Record {
        version: 2,
        interpretation: "Legacy local rules in sequential order; proposed changes, not validated grammatical exclusions. Includes proposal-only one-subject, which ordinary disambiguation does not apply. Event child indices address the flat tree group, not source tokens or bytes.".into(),
        limitations: "Only successful rule updates are recorded, not rejected attempts or proved syntactic attachments. Group snapshots show operational context, not an independently validated premise. Tree W nodes contain ambiguity markers; full original morphological candidates are in source_analysis under its declared matching policy. The lifter uses legacy matching/abbreviation/prosodic machinery and may segment differently. No statistical tagger is run. Replay establishes behavior under the supplied lexicon and current code, not a complete dependency fingerprint or linguistic correctness.".into(),
        source_analysis: analysis_document::Record::from_document(&sentence.analysis_document(MatchPolicy::Exact)),
        input: tree(trace.input()),
        proposed: tree(trace.proposed()),
        events: trace.events().iter().map(|e| Event {
            rule: e.rule.into(), applied_by_default: e.applied_by_default, child: e.child,
            context: e.context.iter().map(tree).collect(), proposed: tree(&e.proposed),
        }).collect(),
    })
}

/// Bound output during serialization rather than after allocating the payload.
pub fn to_json(record: &Record) -> Result<String, Box<dyn Error>> {
    struct Bounded(Vec<u8>);
    impl Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > MAX_RECORD_BYTES.saturating_sub(self.0.len()) {
                return Err(std::io::Error::other(
                    "context trace record exceeds byte limit",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut out = Bounded(Vec::new());
    serde_json::to_writer(&mut out, record)?;
    Ok(String::from_utf8(out.0)?)
}

/// Recompute from the original source; no serialized tree is executed or trusted.
pub fn from_json(bytes: &[u8], lexicon: &Lexicon) -> Result<Record, Box<dyn Error>> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err("context trace record exceeds byte limit".into());
    }
    let record: Record = serde_json::from_slice(bytes)?;
    let expected = evaluate(lexicon, &record.source_analysis.source)?;
    if record != expected {
        return Err("context trace differs from current source/model replay".into());
    }
    Ok(expected)
}
