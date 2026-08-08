use crate::normalize::{checked_tsv, has_wiki_markup};
use crate::schema::Registry;
use old_church_slavonic_core::orthography::{Script, canonical_display, detect_script, lookup_key};
use std::collections::BTreeSet;

pub const MIN_ACCEPTED_LEXEMES: usize = 3_000;
pub const MIN_ACCEPTED_FORMS: usize = 130_000;

pub fn coverage(lexemes: usize, forms: usize) -> Result<(), String> {
    if lexemes < MIN_ACCEPTED_LEXEMES || forms < MIN_ACCEPTED_FORMS {
        Err(format!(
            "registry coverage collapsed below the pinned snapshot floor: {lexemes} lexemes / {forms} forms"
        ))
    } else {
        Ok(())
    }
}

pub fn registry(registry: &Registry) -> Result<(), String> {
    let mut lexeme_ids = BTreeSet::new();
    for row in &registry.lexemes {
        if !lexeme_ids.insert(row.id.as_str()) {
            return Err(format!("duplicate lexeme id: {}", row.id));
        }
        for (value, name) in [
            (&row.id, "lexeme id"),
            (&row.lemma, "lemma"),
            (&row.page_word, "page word"),
            (&row.key, "lookup key"),
            (&row.pos, "part of speech"),
            (&row.head_templates, "head templates"),
        ] {
            if value.is_empty() {
                return Err(format!("{name} is empty for {}", row.id));
            }
            checked_tsv(value, name)?;
        }
        for (value, name) in [
            (&row.class, "normalized class"),
            (&row.raw_class, "raw class"),
            (&row.gender, "gender"),
            (&row.animacy, "animacy"),
            (&row.number_restriction, "number restriction"),
            (&row.signature, "signature"),
        ] {
            checked_tsv(value, name)?;
        }
        for (value, name) in [
            (&row.lemma, "lemma"),
            (&row.page_word, "page word"),
            (&row.key, "lookup key"),
        ] {
            if has_wiki_markup(value) {
                return Err(format!("{name} contains MediaWiki markup for {}", row.id));
            }
        }
        for (value, name) in [(&row.lemma, "lemma"), (&row.page_word, "page word")] {
            if canonical_display(value).map_err(|error| error.to_string())? != *value {
                return Err(format!("{name} is not NFC for {}", row.id));
            }
            if !matches!(detect_script(value), Script::Cyrillic | Script::Glagolitic) {
                return Err(format!("{name} is not an OCS-script word for {}", row.id));
            }
        }
        if lookup_key(&row.key).map_err(|error| error.to_string())? != row.key {
            return Err(format!("lookup key is not normalized for {}", row.id));
        }
        if !registry
            .aliases
            .iter()
            .any(|alias| alias.lexeme_id == row.id && alias.key == row.key)
        {
            return Err(format!(
                "canonical lookup key is not an alias for {}",
                row.id
            ));
        }
    }
    let mut form_keys = BTreeSet::new();
    let mut form_values = BTreeSet::new();
    let mut previous_cell: Option<(&str, &str, u16)> = None;
    for row in &registry.forms {
        if !lexeme_ids.contains(row.lexeme_id.as_str()) {
            return Err(format!("form points at missing lexeme: {}", row.lexeme_id));
        }
        if row.form.is_empty() || matches!(row.form.as_str(), "-" | "—" | "no-table-tags") {
            return Err(format!("sentinel/empty public form for {}", row.lexeme_id));
        }
        if !form_keys.insert((row.lexeme_id.as_str(), row.feature.as_str(), row.rank)) {
            return Err(format!(
                "duplicate form key: {} {} {}",
                row.lexeme_id, row.feature, row.rank
            ));
        }
        if !form_values.insert((
            row.lexeme_id.as_str(),
            row.feature.as_str(),
            row.form.as_str(),
            row.romanization.as_str(),
        )) {
            return Err(format!(
                "duplicate public variant: {} {} {}",
                row.lexeme_id, row.feature, row.form
            ));
        }
        checked_tsv(&row.feature, "feature key")?;
        checked_tsv(&row.form, "public form")?;
        checked_tsv(&row.romanization, "romanization")?;
        if row.source_spelling.is_empty() {
            return Err(format!("source spelling is empty for {}", row.lexeme_id));
        }
        checked_tsv(&row.source_spelling, "source spelling")?;
        checked_tsv(&row.source_tags, "source tags")?;
        if has_wiki_markup(&row.form) {
            return Err(format!(
                "public form contains MediaWiki markup for {}",
                row.lexeme_id
            ));
        }
        if canonical_display(&row.form).map_err(|error| error.to_string())? != row.form {
            return Err(format!("public form is not NFC for {}", row.lexeme_id));
        }
        if !matches!(
            detect_script(&row.form),
            Script::Cyrillic | Script::Glagolitic
        ) {
            return Err(format!(
                "public form is not an OCS-script word for {}",
                row.lexeme_id
            ));
        }
        if let Some((id, feature, rank)) = previous_cell {
            if id == row.lexeme_id && feature == row.feature && row.rank != rank + 1 {
                return Err(format!(
                    "non-contiguous variant ranks for {} {}: {} follows {}",
                    row.lexeme_id, row.feature, row.rank, rank
                ));
            }
        }
        if row.rank != 0
            && previous_cell
                .is_none_or(|(id, feature, _)| id != row.lexeme_id || feature != row.feature)
        {
            return Err(format!(
                "first variant rank is not zero for {} {}",
                row.lexeme_id, row.feature
            ));
        }
        previous_cell = Some((&row.lexeme_id, &row.feature, row.rank));
    }
    let mut alias_keys = BTreeSet::new();
    for row in &registry.aliases {
        if !lexeme_ids.contains(row.lexeme_id.as_str()) {
            return Err(format!("alias points at missing lexeme: {}", row.lexeme_id));
        }
        checked_tsv(&row.key, "alias key")?;
        if row.source_spellings.is_empty() {
            return Err(format!("alias source spelling is empty: {}", row.key));
        }
        checked_tsv(&row.source_spellings, "alias source spellings")?;
        if has_wiki_markup(&row.key) || has_wiki_markup(&row.source_spellings) {
            return Err(format!("alias contains MediaWiki markup: {}", row.key));
        }
        if !alias_keys.insert((row.key.as_str(), row.lexeme_id.as_str())) {
            return Err(format!("duplicate alias: {} -> {}", row.key, row.lexeme_id));
        }
    }
    Ok(())
}

pub fn noun_citations(registry: &Registry, exemptions: &BTreeSet<String>) -> Result<(), String> {
    let mut missing = Vec::new();
    let mut used_exemptions = BTreeSet::new();
    for lexeme in registry
        .lexemes
        .iter()
        .filter(|lexeme| lexeme.pos == "noun")
        .filter(|lexeme| !matches!(lexeme.number_restriction.as_str(), "du" | "pl"))
    {
        let nominatives = registry
            .forms
            .iter()
            .filter(|form| form.lexeme_id == lexeme.id && form.feature == "noun:nom:sg");
        let mut found_cell = false;
        let mut found_citation = false;
        for form in nominatives {
            found_cell = true;
            found_citation |= form.form == lexeme.lemma
                || old_church_slavonic_core::orthography::lookup_key(&form.form)
                    .is_ok_and(|key| key == lexeme.key);
        }
        if found_cell && !found_citation {
            if exemptions.contains(&lexeme.id) {
                used_exemptions.insert(lexeme.id.as_str());
            } else {
                missing.push(lexeme.id.as_str());
            }
        }
    }
    let unused = exemptions
        .iter()
        .filter(|id| !used_exemptions.contains(id.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    if !unused.is_empty() {
        Err(format!(
            "unused or stale noun citation exemptions: {}",
            unused.join(", ")
        ))
    } else if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "noun nominative singular omits canonical citations for {}; add sourced exemptions if intentional",
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AliasRow, FormRow, LexemeRow};

    fn valid_registry() -> Registry {
        Registry {
            lexemes: vec![LexemeRow {
                id: "слово|noun|fixture".to_string(),
                lemma: "слово".to_string(),
                page_word: "слово".to_string(),
                key: "слово".to_string(),
                pos: "noun".to_string(),
                class: "o-n-hard".to_string(),
                raw_class: "o-stem".to_string(),
                gender: "n".to_string(),
                animacy: String::new(),
                number_restriction: String::new(),
                head_templates: "[]".to_string(),
                signature: "fixture".to_string(),
            }],
            aliases: vec![AliasRow {
                key: "слово".to_string(),
                lexeme_id: "слово|noun|fixture".to_string(),
                source_spellings: "canonical:\"слово\"".to_string(),
            }],
            forms: vec![FormRow {
                lexeme_id: "слово|noun|fixture".to_string(),
                feature: "noun:nom:sg".to_string(),
                rank: 0,
                form: "слово".to_string(),
                romanization: "slovo".to_string(),
                source_spelling: "слово".to_string(),
                source_tags: "nominative,singular".to_string(),
            }],
        }
    }

    #[test]
    fn duplicate_rank_witness_is_rejected() {
        let mut fixture = valid_registry();
        fixture.forms.push(fixture.forms[0].clone());
        assert!(
            registry(&fixture)
                .expect_err("duplicate rank must fail")
                .contains("duplicate form key")
        );
    }

    #[test]
    fn sentinel_witness_is_rejected() {
        let mut fixture = valid_registry();
        fixture.forms[0].form = "—".to_string();
        assert!(
            registry(&fixture)
                .expect_err("sentinel must fail")
                .contains("sentinel")
        );
    }

    #[test]
    fn mediawiki_markup_witness_is_rejected() {
        let mut fixture = valid_registry();
        fixture.forms[0].form = "сло{{{2}}}во".to_string();
        assert!(
            registry(&fixture)
                .expect_err("template placeholder must fail")
                .contains("MediaWiki markup")
        );
    }

    #[test]
    fn non_nfc_public_form_witness_is_rejected() {
        let mut fixture = valid_registry();
        fixture.forms[0].form = "И\u{306}".to_string();
        assert!(
            registry(&fixture)
                .expect_err("decomposed public form must fail")
                .contains("not NFC")
        );
    }

    #[test]
    fn citation_witness_needs_an_explicit_exemption() {
        let mut fixture = valid_registry();
        fixture.forms[0].form = "словесе".to_string();
        assert!(
            noun_citations(&fixture, &BTreeSet::new())
                .expect_err("citation mismatch must fail")
                .contains("omits canonical")
        );
        let exemptions = BTreeSet::from(["слово|noun|fixture".to_string()]);
        noun_citations(&fixture, &exemptions).expect("explicit exemption is consumed");
    }

    #[test]
    fn coverage_floor_witness_is_rejected() {
        assert!(
            coverage(MIN_ACCEPTED_LEXEMES - 1, MIN_ACCEPTED_FORMS)
                .expect_err("one missing lexeme must fail")
                .contains("coverage collapsed")
        );
        assert!(coverage(MIN_ACCEPTED_LEXEMES, MIN_ACCEPTED_FORMS).is_ok());
    }
}
