use old_church_slavonic_core::orthography;

pub fn lookup_key(value: &str) -> Result<String, String> {
    orthography::lookup_key(value).map_err(|error| error.to_string())
}

pub fn checked_tsv(value: &str, field: &str) -> Result<(), String> {
    if value.contains(['\t', '\n', '\r']) {
        Err(format!("{field} contains a TSV delimiter"))
    } else {
        Ok(())
    }
}

/// Reject source-rendering failures before they become public word forms.
pub fn has_wiki_markup(value: &str) -> bool {
    ["{{", "}}", "[[", "]]", "<", ">"]
        .into_iter()
        .any(|marker| value.contains(marker))
}
