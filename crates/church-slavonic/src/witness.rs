//! Immutable source text and validated UTF-8 spans, independent of analysis.

use sha2::{Digest, Sha256};
use std::ops::Range;

/// A source address is supplied by ingestion; it is not inferred from a lemma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAddress {
    pub witness: String,
    pub document: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    source: String,
    address: Option<SourceAddress>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    Text,
    Whitespace,
}

/// A view borrows its witness so it cannot accidentally resolve against another
/// source. Its validated range and kind cannot be mutated by a caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan<'a> {
    witness: &'a Witness,
    range: Range<usize>,
}

impl SourceSpan<'_> {
    pub fn text(&self) -> &str {
        &self.witness.source[self.range.clone()]
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn address(&self) -> Option<&SourceAddress> {
        self.witness.address.as_ref()
    }
}

impl Witness {
    pub fn new(source: impl Into<String>, address: Option<SourceAddress>) -> Self {
        Self {
            source: source.into(),
            address,
        }
    }

    pub fn sha256(&self) -> String {
        format!("{:x}", Sha256::digest(self.source.as_bytes()))
    }

    pub fn reproduce(&self) -> &str {
        &self.source
    }
    pub fn address(&self) -> Option<&SourceAddress> {
        self.address.as_ref()
    }

    /// Reject reversed, out-of-bounds, and mid-codepoint ranges.
    pub fn span(&self, range: Range<usize>) -> Option<SourceSpan<'_>> {
        self.source.get(range.clone())?;
        Some(SourceSpan {
            witness: self,
            range,
        })
    }

    /// Partition the original bytes without claiming a linguistic tokenization.
    /// Punctuation remains in text spans. Whitespace is never discarded.
    pub fn segments(&self) -> Vec<(SegmentKind, SourceSpan<'_>)> {
        let mut out = Vec::new();
        let mut start = 0;
        let mut previous = None;
        for (offset, ch) in self.source.char_indices() {
            let kind = if ch.is_whitespace() {
                SegmentKind::Whitespace
            } else {
                SegmentKind::Text
            };
            if let Some(old) = previous
                && old != kind
            {
                out.push((
                    old,
                    SourceSpan {
                        witness: self,
                        range: start..offset,
                    },
                ));
                start = offset;
            }
            previous = Some(kind);
        }
        if let Some(kind) = previous {
            out.push((
                kind,
                SourceSpan {
                    witness: self,
                    range: start..self.source.len(),
                },
            ));
        }
        out
    }
}
