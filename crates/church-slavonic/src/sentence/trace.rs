//! Bounded replay records for legacy rule proposals, not validated exclusions.
use super::{Node, TreeError, rules};
use crate::Lexicon;

pub const MAX_EVENTS: usize = 128;
pub const MAX_TREE_NODES: usize = 256;
pub const MAX_TREE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub rule: &'static str,
    /// Operational status only; true does not imply a validated linguistic rule.
    pub applied_by_default: bool,
    /// Direct child of the flat input group; not a source byte/token offset.
    pub child: usize,
    /// Entire current group before this decision, including prior proposals.
    pub context: Vec<Node>,
    pub proposed: Node,
}

#[derive(Debug, Clone)]
pub struct Trace {
    input: Node,
    proposed: Node,
    events: Vec<Event>,
}
impl Trace {
    pub fn input(&self) -> &Node {
        &self.input
    }
    pub fn proposed(&self) -> &Node {
        &self.proposed
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Recorder {
    pub events: Vec<Event>,
    pub exhausted: bool,
}

pub(crate) fn check(nodes: &[Node]) -> Result<(), TreeError> {
    if nodes.len() > MAX_TREE_NODES {
        return Err(limit());
    }
    let mut stack: Vec<_> = nodes.iter().map(|n| (n, 0)).collect();
    let mut count = 0;
    let mut bytes = 0usize;
    while let Some((node, depth)) = stack.pop() {
        count += 1;
        if count > MAX_TREE_NODES || depth > 32 {
            return Err(limit());
        }
        if let Node::W { notes, .. } | Node::Lex { notes, .. } = node {
            if notes.len() > MAX_TREE_BYTES / 64 {
                return Err(limit());
            }
            bytes = bytes.saturating_add(notes.len().saturating_mul(64));
        }
        let mut add = |s: &str| {
            bytes = bytes.saturating_add(s.len());
        };
        match node {
            Node::W { surface, notes } => {
                add(surface);
                for (k, v) in notes {
                    add(k);
                    add(v);
                }
            }
            Node::Lex {
                id, notes, cells, ..
            } => {
                add(id);
                for (k, v) in notes {
                    add(k);
                    add(v);
                }
                bytes = bytes.saturating_add(cells.len().saturating_mul(32));
            }
            Node::Punct(s) | Node::Fn(s) => add(s),
            Node::Cap(child) => stack.push((child, depth + 1)),
            Node::Abbr {
                prefix,
                full,
                child,
            } => {
                add(prefix);
                if let Some(s) = full {
                    add(s);
                }
                stack.push((child, depth + 1));
            }
            Node::Pw {
                host, enclitics, ..
            } => {
                stack.push((host, depth + 1));
                if enclitics.len() > MAX_TREE_NODES {
                    return Err(limit());
                }
                stack.extend(enclitics.iter().map(|n| (n, depth + 1)));
            }
            Node::Group { head, children } => {
                if depth > 0 {
                    return Err(TreeError("nested contextual groups are unsupported".into()));
                }
                add(head);
                if children.len() > MAX_TREE_NODES {
                    return Err(limit());
                }
                stack.extend(children.iter().map(|n| (n, depth + 1)));
            }
        }
        if bytes > MAX_TREE_BYTES {
            return Err(limit());
        }
    }
    Ok(())
}
fn limit() -> TreeError {
    TreeError("context trace limits exceeded; no complete proposal returned".into())
}

/// Evaluate a copy. Repeated calls start from the same input, not prior output.
/// Rules follow their legacy sequential schedule, including proposal-only
/// `one-subject`, which ordinary disambiguation does not apply. Snapshots expose that schedule
/// and the state read at each change; they do not establish grammatical premises.
pub fn evaluate(input: &Node, lexicon: &Lexicon) -> Result<Trace, TreeError> {
    check(std::slice::from_ref(input))?;
    if !matches!(input, Node::Group { .. }) {
        return Err(TreeError("context trace requires a flat group".into()));
    }
    let mut proposed = input.clone();
    let stats = rules::disambiguate_recorded(&mut proposed, lexicon);
    let recorder = stats.recorder.ok_or_else(limit)?;
    if recorder.exhausted {
        return Err(limit());
    }
    check(std::slice::from_ref(&proposed))?;
    Ok(Trace {
        input: input.clone(),
        proposed,
        events: recorder.events,
    })
}
