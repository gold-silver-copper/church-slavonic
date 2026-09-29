//! Original corpus records, separate from mapped inflection labels.
use church_slavonic::witness::Witness;
use serde::Serialize;
use std::{
    io::{self, Write},
    ops::Range,
};

#[derive(Debug, serde::Serialize)]
pub struct ExcludedDocument {
    pub path: String,
    pub sha256: String,
    pub bytes: usize,
    pub reason: String,
}

#[derive(Debug)]
pub struct SourceDocument {
    path: String,
    sha256: String,
    witness: Witness,
}
impl SourceDocument {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn text(&self) -> &str {
        self.witness.reproduce()
    }
    pub fn new(path: String, text: String) -> Result<Self, String> {
        if text.len() > 64 * 1024 * 1024 || path.len() > 4096 {
            return Err("source document size limit".into());
        }
        let witness = Witness::new(text, None);
        Ok(Self {
            path,
            sha256: witness.sha256(),
            witness,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Comment,
    Separator,
    Word,
    Multiword,
    EmptyNode,
    Malformed,
    XmlMarkup,
    XmlToken,
    OtherLanguageToken,
}
#[derive(Debug, Clone, Serialize)]
pub struct SourceRecord {
    pub source: usize,
    pub start: usize,
    pub end: usize,
    pub kind: RecordKind,
    pub mapped_slots: Vec<usize>,
    pub mapping_issue: Option<String>,
}
impl SourceRecord {
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}
#[derive(Debug, Default)]
pub struct Observations {
    documents: Vec<SourceDocument>,
    records: Vec<SourceRecord>,
    bytes: usize,
}
impl Observations {
    pub fn documents(&self) -> &[SourceDocument] {
        &self.documents
    }
    pub fn records(&self) -> &[SourceRecord] {
        &self.records
    }
    pub fn raw(&self, record: &SourceRecord) -> Option<&str> {
        self.documents
            .get(record.source)?
            .text()
            .get(record.range())
    }
    pub fn append(
        &mut self,
        document: SourceDocument,
        mut records: Vec<SourceRecord>,
    ) -> Result<usize, String> {
        let retained = records
            .iter()
            .try_fold(
                document.text().len() + document.path().len() + 128,
                |total, record| {
                    if record.end.saturating_sub(record.start) > 64 * 1024
                        || record.mapped_slots.len() > 128
                        || record
                            .mapping_issue
                            .as_ref()
                            .is_some_and(|s| s.len() > 4096)
                    {
                        return None;
                    }
                    total
                        .checked_add(std::mem::size_of::<SourceRecord>())?
                        .checked_add(record.mapped_slots.len() * std::mem::size_of::<usize>())?
                        .checked_add(record.mapping_issue.as_ref().map_or(0, String::len))
                },
            )
            .ok_or("source record size limit")?;
        if self.bytes.saturating_add(retained) > 128 * 1024 * 1024
            || self.records.len().saturating_add(records.len()) > 1_000_000
            || self.documents.len() >= 10_000
        {
            return Err("corpus observation limit".into());
        }
        let source = self.documents.len();
        let mut cursor = 0;
        for record in &mut records {
            if record.start != cursor
                || record.end <= record.start
                || document.text().get(record.range()).is_none()
            {
                return Err("source records must partition original UTF-8 bytes".into());
            }
            record.source = source;
            cursor = record.end;
        }
        if cursor != document.text().len() {
            return Err("source records omit bytes".into());
        }
        let first = self.records.len();
        self.bytes += retained;
        self.documents.push(document);
        self.records.extend(records);
        Ok(first)
    }
    /// JSONL preserves every original record including its separator bytes.
    /// Consumers can concatenate raw fields per source to reproduce the file.
    pub fn write_jsonl(&self, mut out: impl Write) -> io::Result<()> {
        for (id, document) in self.documents.iter().enumerate() {
            serde_json::to_writer(
                &mut out,
                &serde_json::json!({"schema":1,"type":"source","id":id,"path":document.path(),"sha256":document.sha256(),"bytes":document.text().len()}),
            )?;
            writeln!(out)?;
        }
        for (id, record) in self.records.iter().enumerate() {
            serde_json::to_writer(
                &mut out,
                &serde_json::json!({"schema":1,"type":"record","id":id,"record":record,"raw":self.raw(record).ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"invalid source span"))?}),
            )?;
            writeln!(out)?;
        }
        Ok(())
    }
}
