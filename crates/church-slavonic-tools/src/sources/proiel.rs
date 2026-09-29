//! Event-based PROIEL intake with exact source records; cell mappings remain
//! legacy assertions. Missing surfaces are records, never invented input words.
use super::{
    observations::{RecordKind, SourceDocument, SourceRecord},
    ud::{Corpus, SequenceToken, clean_surface, proiel_token},
};
use quick_xml::{Reader, events::Event};
use std::{collections::BTreeMap, error::Error};

fn gap(
    records: &mut Vec<SourceRecord>,
    text: &str,
    mut start: usize,
    end: usize,
) -> Result<(), Box<dyn Error>> {
    while start < end {
        if records.len() >= 1_000_000 {
            return Err("XML record limit".into());
        }
        let mut stop = end.min(start + 65536);
        while !text.is_char_boundary(stop) {
            stop -= 1;
        }
        records.push(SourceRecord {
            source: 0,
            start,
            end: stop,
            kind: RecordKind::XmlMarkup,
            mapped_slots: vec![],
            mapping_issue: None,
        });
        start = stop;
    }
    Ok(())
}

pub fn append_xml(corpus: &mut Corpus, path: String, text: String) -> Result<(), Box<dyn Error>> {
    let document = SourceDocument::new(path, text)?;
    let xml = document.text();
    let mut reader = Reader::from_str(xml);
    let mut language = None;
    let mut selected = false;
    let mut in_sentence = false;
    let mut sentence = Vec::new();
    let mut parsed = Corpus::default();
    let mut records = Vec::new();
    let mut cursor = 0;
    let mut depth = 0usize;
    let mut roots = 0;
    let mut declared = false;
    let mut token_depth = None;
    let slot_base = corpus.slots.len();
    let record_base = corpus.observations.records().len();
    loop {
        let start = reader.buffer_position() as usize;
        let event = reader.read_event()?;
        let end = reader.buffer_position() as usize;
        match event {
            Event::Decl(declaration) => {
                if declared || roots != 0 {
                    return Err("misplaced or duplicate XML declaration".into());
                }
                declared = true;
                if declaration.version()?.as_ref() != b"1.0" {
                    return Err("only XML 1.0 is supported".into());
                }
                if let Some(encoding) = declaration.encoding()
                    && !encoding?.eq_ignore_ascii_case(b"utf-8")
                {
                    return Err("only UTF-8 XML is supported".into());
                }
            }
            Event::DocType(_) => {
                return Err(
                    "XML DTDs are unsupported; external entities are never resolved".into(),
                );
            }
            Event::Start(ref tag) | Event::Empty(ref tag) => {
                let empty = matches!(event, Event::Empty(_));
                if depth == 0 {
                    if roots != 0 || tag.name().as_ref() != b"proiel" {
                        return Err("expected one proiel root".into());
                    }
                    roots += 1;
                }
                if !empty {
                    depth += 1;
                }
                if depth > 128 || end - start > 65536 {
                    return Err("XML nesting or tag size limit".into());
                }
                let mut attrs = BTreeMap::new();
                for attr in tag.attributes() {
                    let attr = attr?;
                    let key = std::str::from_utf8(attr.key.as_ref())?.to_string();
                    let value = attr
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Implicit1_0,
                            reader.decoder(),
                        )?
                        .into_owned();
                    attrs.insert(key, value);
                }
                match tag.name().as_ref() {
                    b"source" => {
                        if language.is_some() {
                            return Err("nested XML source".into());
                        }
                        let lang = attrs.get("language").cloned().unwrap_or_default();
                        selected |= lang == "chu";
                        if !empty {
                            language = Some(lang);
                        }
                    }
                    b"sentence" => {
                        if in_sentence {
                            return Err("nested XML sentence".into());
                        }
                        in_sentence = !empty;
                    }
                    b"token" => {
                        if token_depth.is_some() {
                            return Err("nested XML token".into());
                        }
                        if !empty {
                            token_depth = Some(depth);
                        }
                        gap(&mut records, xml, cursor, start)?;
                        let mut record = SourceRecord {
                            source: 0,
                            start,
                            end,
                            kind: RecordKind::XmlToken,
                            mapped_slots: vec![],
                            mapping_issue: None,
                        };
                        if language.as_deref() != Some("chu") {
                            record.kind = RecordKind::OtherLanguageToken;
                            record.mapping_issue = Some("outside selected source language".into());
                        } else if !in_sentence {
                            return Err("XML token outside sentence".into());
                        } else if let Some(form) = attrs.get("form").filter(|s| !s.is_empty()) {
                            parsed.tokens += 1;
                            let before = parsed.slots.len();
                            let old_skips = parsed.skipped.clone();
                            if let (Some(lemma), Some(pos), Some(morphology)) = (
                                attrs.get("lemma"),
                                attrs.get("part-of-speech"),
                                attrs.get("morphology"),
                            ) {
                                proiel_token(&mut parsed, form, lemma, pos, morphology);
                            } else {
                                parsed.skip("missing token annotation");
                            }
                            record.mapped_slots = (before..parsed.slots.len())
                                .map(|i| i + slot_base)
                                .collect();
                            record.mapping_issue = parsed
                                .skipped
                                .iter()
                                .find(|(k, v)| **v > old_skips.get(*k).copied().unwrap_or(0))
                                .map(|(k, _)| k.to_string());
                            if record.mapped_slots.is_empty() && record.mapping_issue.is_none() {
                                parsed.skip("no mapped cells");
                                record.mapping_issue = Some("no mapped cells".into());
                            }
                            sentence.push(SequenceToken {
                                surface: clean_surface(form),
                                lemma: attrs
                                    .get("lemma")
                                    .map(|s| s.to_lowercase())
                                    .unwrap_or_else(|| "_".into()),
                                object: attrs.get("relation").is_some_and(|s| s == "obj"),
                                slots: record.mapped_slots.clone(),
                                source_record: Some(record_base + records.len()),
                            });
                        } else {
                            record.mapping_issue = Some("no source surface".into());
                        }
                        if records.len() >= 1_000_000 {
                            return Err("XML record limit".into());
                        }
                        records.push(record);
                        cursor = end;
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                depth = depth.checked_sub(1).ok_or("unexpected XML closing tag")?;
                match tag.name().as_ref() {
                    b"token" => {
                        token_depth = None;
                    }
                    b"sentence" => {
                        if !sentence.is_empty() {
                            parsed.sentences.push(std::mem::take(&mut sentence));
                        }
                        in_sentence = false;
                    }
                    b"source" => {
                        language = None;
                    }
                    _ => {}
                }
            }
            Event::Text(text)
                if depth == 0 && !text.as_ref().iter().all(u8::is_ascii_whitespace) =>
            {
                return Err("text outside XML root".into());
            }
            Event::CData(_) | Event::GeneralRef(_) if depth == 0 => {
                return Err("content outside XML root".into());
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if roots != 1 || depth != 0 || in_sentence || language.is_some() || token_depth.is_some() {
        return Err("truncated XML structure".into());
    }
    if !selected {
        if corpus.excluded_sources.len() >= 10_000 {
            return Err("excluded source count limit".into());
        }
        corpus
            .excluded_sources
            .push(super::observations::ExcludedDocument {
                path: document.path().into(),
                sha256: document.sha256().into(),
                bytes: document.text().len(),
                reason: "no source element with language chu".into(),
            });
        return Ok(());
    }
    gap(&mut records, xml, cursor, xml.len())?;
    corpus.observations.append(document, records)?;
    corpus.tokens += parsed.tokens;
    corpus.slots.extend(parsed.slots);
    corpus.sentences.extend(parsed.sentences);
    for (reason, count) in parsed.skipped {
        *corpus.skipped.entry(reason).or_default() += count;
    }
    Ok(())
}
