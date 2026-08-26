use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExtractionReport {
    pub schema_version: u32,
    pub input_lines: usize,
    pub parse_failures: usize,
    pub ocs_entries: usize,
    pub accepted_lexemes: usize,
    pub accepted_forms: usize,
    pub accepted_by_pos: BTreeMap<String, usize>,
    pub accepted_by_class: BTreeMap<String, usize>,
    pub accepted_by_feature: BTreeMap<String, usize>,
    pub accepted_tag_signatures: BTreeMap<String, usize>,
    pub rejected_tag_signatures: BTreeMap<String, usize>,
    pub dropped_by_reason: BTreeMap<String, usize>,
    pub scripts: BTreeMap<String, usize>,
    pub accepted_by_source: BTreeMap<String, usize>,
    pub ambiguous_lemma_pos_pairs: usize,
    pub lexeme_ids_added: usize,
    pub lexeme_ids_removed: usize,
    pub added_lexeme_ids: Vec<String>,
    pub removed_lexeme_ids: Vec<String>,
}

impl ExtractionReport {
    pub fn markdown(&self) -> String {
        let mut out = String::from("# Extraction coverage\n\n");
        out.push_str(&format!("Schema: {}\n\n", self.schema_version));
        out.push_str(&format!("- input lines: {}\n", self.input_lines));
        out.push_str(&format!("- parse failures: {}\n", self.parse_failures));
        out.push_str(&format!("- OCS entries: {}\n", self.ocs_entries));
        out.push_str(&format!("- accepted lexemes: {}\n", self.accepted_lexemes));
        out.push_str(&format!("- accepted forms: {}\n\n", self.accepted_forms));
        out.push_str(&format!(
            "- ambiguous lemma/POS lookup pairs: {}\n\n",
            self.ambiguous_lemma_pos_pairs
        ));
        out.push_str(&format!(
            "- lexeme IDs added since prior registry: {}\n",
            self.lexeme_ids_added
        ));
        out.push_str(&format!(
            "- lexeme IDs removed since prior registry: {}\n\n",
            self.lexeme_ids_removed
        ));
        if !self.added_lexeme_ids.is_empty() {
            out.push_str("Added IDs:\n\n");
            for id in &self.added_lexeme_ids {
                out.push_str(&format!("- `{id}`\n"));
            }
            out.push('\n');
        }
        if !self.removed_lexeme_ids.is_empty() {
            out.push_str("Removed IDs:\n\n");
            for id in &self.removed_lexeme_ids {
                out.push_str(&format!("- `{id}`\n"));
            }
            out.push('\n');
        }
        table(
            &mut out,
            "Accepted by part of speech",
            &self.accepted_by_pos,
        );
        table(&mut out, "Accepted by class", &self.accepted_by_class);
        table(&mut out, "Accepted by cell type", &self.accepted_by_feature);
        table(&mut out, "Accepted by source", &self.accepted_by_source);
        table(
            &mut out,
            "Accepted tag signatures",
            &self.accepted_tag_signatures,
        );
        table(
            &mut out,
            "Rejected tag signatures",
            &self.rejected_tag_signatures,
        );
        table(&mut out, "Dropped by reason", &self.dropped_by_reason);
        table(&mut out, "Scripts", &self.scripts);
        while out.ends_with("\n\n") {
            out.pop();
        }
        out
    }
}

fn table(out: &mut String, title: &str, values: &BTreeMap<String, usize>) {
    out.push_str(&format!("## {title}\n\n| value | count |\n|---|---:|\n"));
    for (value, count) in values {
        out.push_str(&format!("| `{value}` | {count} |\n"));
    }
    out.push('\n');
}
