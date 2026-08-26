use crate::ocs::normalize::{checked_tsv, has_wiki_markup};
use crate::ocs::schema::Registry;
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
    overrides(registry, &lexeme_ids)?;
    verb_metadata(registry, &lexeme_ids)?;
    Ok(())
}

fn overrides(registry: &Registry, lexeme_ids: &BTreeSet<&str>) -> Result<(), String> {
    let pos_by_id = registry
        .lexemes
        .iter()
        .map(|row| (row.id.as_str(), row.pos.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let source_cells = registry
        .forms
        .iter()
        .map(|row| (row.lexeme_id.as_str(), row.feature.as_str()))
        .collect::<BTreeSet<_>>();
    let mut keys = BTreeSet::new();
    let mut values = BTreeSet::new();
    let mut previous: Option<(&str, &str, u16)> = None;
    for row in &registry.overrides {
        if !lexeme_ids.contains(row.lexeme_id.as_str()) {
            return Err(format!(
                "override points at missing lexeme: {}",
                row.lexeme_id
            ));
        }
        if row.feature.is_empty() || row.reason.trim().is_empty() || row.authority.trim().is_empty()
        {
            return Err(format!(
                "override has an empty feature, reason, or authority for {}",
                row.lexeme_id
            ));
        }
        let pos = pos_by_id
            .get(row.lexeme_id.as_str())
            .ok_or_else(|| format!("override has no lexeme POS for {}", row.lexeme_id))?;
        if !valid_override_feature(pos, &row.feature) {
            return Err(format!(
                "override has an invalid feature for {pos}: {}",
                row.feature
            ));
        }
        if source_cells.contains(&(row.lexeme_id.as_str(), row.feature.as_str())) {
            return Err(format!(
                "override would shadow an exact source cell: {} {}",
                row.lexeme_id, row.feature
            ));
        }
        if row.form.is_empty() || matches!(row.form.as_str(), "-" | "—" | "no-table-tags") {
            return Err(format!(
                "override contains a sentinel/empty form for {}",
                row.lexeme_id
            ));
        }
        for (value, name) in [
            (&row.feature, "override feature"),
            (&row.form, "override form"),
            (&row.romanization, "override romanization"),
            (&row.reason, "override reason"),
            (&row.authority, "override authority"),
        ] {
            checked_tsv(value, name)?;
        }
        if has_wiki_markup(&row.form) {
            return Err(format!(
                "override contains MediaWiki markup for {}",
                row.lexeme_id
            ));
        }
        if canonical_display(&row.form).map_err(|error| error.to_string())? != row.form {
            return Err(format!("override form is not NFC for {}", row.lexeme_id));
        }
        if !matches!(
            detect_script(&row.form),
            Script::Cyrillic | Script::Glagolitic
        ) {
            return Err(format!(
                "override form is not OCS script for {}",
                row.lexeme_id
            ));
        }
        if !keys.insert((row.lexeme_id.as_str(), row.feature.as_str(), row.rank)) {
            return Err(format!(
                "duplicate override key: {} {} {}",
                row.lexeme_id, row.feature, row.rank
            ));
        }
        if !values.insert((
            row.lexeme_id.as_str(),
            row.feature.as_str(),
            row.form.as_str(),
        )) {
            return Err(format!(
                "duplicate override variant: {} {} {}",
                row.lexeme_id, row.feature, row.form
            ));
        }
        if row.rank == 0 {
            previous = Some((&row.lexeme_id, &row.feature, row.rank));
        } else if previous.is_none_or(|(id, feature, rank)| {
            id != row.lexeme_id || feature != row.feature || row.rank != rank + 1
        }) {
            return Err(format!(
                "non-contiguous override ranks for {} {}",
                row.lexeme_id, row.feature
            ));
        } else {
            previous = Some((&row.lexeme_id, &row.feature, row.rank));
        }
    }
    Ok(())
}

fn valid_override_feature(pos: &str, feature: &str) -> bool {
    let parts = feature.split(':').collect::<Vec<_>>();
    match (pos, parts.as_slice()) {
        ("noun", ["noun", case, number]) => valid_case(case) && valid_number(number),
        ("adj", ["adj", "comparative", "citation"]) => true,
        ("adj", ["adj", form, case, number, gender, animacy]) => {
            matches!(*form, "short" | "long")
                && valid_case(case)
                && valid_number(number)
                && valid_gender(gender)
                && matches!(*animacy, "an" | "in")
        }
        ("verb", ["verb", "finite", tense, person, number]) => {
            matches!(*tense, "present" | "imperfect" | "aorist")
                && valid_person(person)
                && valid_number(number)
        }
        ("verb", ["verb", "imperative", person, number]) => {
            matches!(
                (*person, *number),
                ("2" | "3", "sg") | ("1" | "2", "du" | "pl")
            )
        }
        ("verb", ["verb", "l-participle", gender, number]) => {
            valid_gender(gender) && valid_number(number)
        }
        ("verb", ["verb", "participle", kind, "citation"]) => valid_participle_kind(kind),
        (
            "verb",
            [
                "verb",
                "participle",
                kind,
                "adj",
                form,
                case,
                number,
                gender,
                animacy,
            ],
        ) => {
            valid_participle_kind(kind)
                && matches!(*form, "short" | "long")
                && valid_case(case)
                && valid_number(number)
                && valid_gender(gender)
                && matches!(*animacy, "an" | "in")
        }
        ("verb", ["verb", "infinitive" | "supine" | "verbal-noun"]) => true,
        ("pron" | "num" | "det", ["decl", decl_pos, case, number]) => {
            *decl_pos == pos && valid_case(case) && valid_number(number)
        }
        ("pron" | "num" | "det", ["decl", decl_pos, case, number, dimension]) => {
            *decl_pos == pos
                && valid_case(case)
                && valid_number(number)
                && (valid_gender(dimension) || valid_person(dimension))
        }
        _ => false,
    }
}

fn valid_case(value: &str) -> bool {
    matches!(value, "nom" | "gen" | "dat" | "acc" | "ins" | "loc" | "voc")
}

fn valid_number(value: &str) -> bool {
    matches!(value, "sg" | "du" | "pl")
}

fn valid_gender(value: &str) -> bool {
    matches!(value, "m" | "f" | "n")
}

fn valid_person(value: &str) -> bool {
    matches!(value, "1" | "2" | "3")
}

fn valid_participle_kind(value: &str) -> bool {
    matches!(
        value,
        "present-active" | "present-passive" | "past-active" | "past-passive"
    )
}

fn verb_metadata(registry: &Registry, lexeme_ids: &BTreeSet<&str>) -> Result<(), String> {
    let verb_ids = registry
        .lexemes
        .iter()
        .filter(|row| row.pos == "verb")
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let form_cells = registry
        .forms
        .iter()
        .map(|row| (row.lexeme_id.as_str(), row.feature.as_str()))
        .collect::<BTreeSet<_>>();
    let form_values = registry
        .forms
        .iter()
        .map(|row| {
            (
                row.lexeme_id.as_str(),
                row.feature.as_str(),
                row.form.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut keys = BTreeSet::new();
    let mut groups: std::collections::BTreeMap<
        (&str, &str, u16),
        std::collections::BTreeMap<&str, &str>,
    > = std::collections::BTreeMap::new();
    let mut ranks: std::collections::BTreeMap<(&str, &str), BTreeSet<u16>> =
        std::collections::BTreeMap::new();
    for row in &registry.verb_metadata {
        if !lexeme_ids.contains(row.lexeme_id.as_str()) {
            return Err(format!(
                "verb metadata points at missing lexeme: {}",
                row.lexeme_id
            ));
        }
        if !verb_ids.contains(row.lexeme_id.as_str()) {
            return Err(format!(
                "verb metadata points at a non-verb lexeme: {}",
                row.lexeme_id
            ));
        }
        if !keys.insert((
            row.lexeme_id.as_str(),
            row.system.as_str(),
            row.analysis_rank,
            row.field.as_str(),
        )) {
            return Err(format!(
                "duplicate verb metadata field: {} {} {} {}",
                row.lexeme_id, row.system, row.analysis_rank, row.field
            ));
        }
        for (value, name) in [
            (&row.system, "verb metadata system"),
            (&row.field, "verb metadata field"),
            (&row.value, "verb metadata value"),
            (&row.provenance, "verb metadata provenance"),
            (&row.source_feature, "verb metadata source feature"),
            (&row.source_form, "verb metadata source form"),
            (&row.crosscheck_features, "verb metadata cross-checks"),
            (&row.authority, "verb metadata authority"),
        ] {
            checked_tsv(value, name)?;
        }
        if row.value.is_empty()
            || row.provenance.is_empty()
            || row.source_feature.is_empty()
            || row.source_form.is_empty()
            || row.authority.is_empty()
        {
            return Err(format!("empty verb metadata field for {}", row.lexeme_id));
        }
        if !matches!(
            row.provenance.as_str(),
            "dictionary-principal-part"
                | "dictionary-headword-metadata"
                | "curated-grammar-override"
        ) {
            return Err(format!(
                "unknown verb metadata provenance {} for {}",
                row.provenance, row.lexeme_id
            ));
        }
        validate_metadata_code(&row.system, &row.field, &row.value)?;
        if row.field == "stem"
            || row.field == "first-singular-stem"
            || row.field == "second-third-singular"
            || !row.source_feature.starts_with("headword:")
        {
            if canonical_display(&row.source_form).map_err(|error| error.to_string())?
                != row.source_form
            {
                return Err(format!(
                    "verb metadata source form is not NFC for {}",
                    row.lexeme_id
                ));
            }
            if !matches!(
                detect_script(&row.source_form),
                Script::Cyrillic | Script::Glagolitic
            ) {
                return Err(format!(
                    "verb metadata source form is not OCS script for {}",
                    row.lexeme_id
                ));
            }
        }
        if matches!(
            row.field.as_str(),
            "stem" | "first-singular-stem" | "second-third-singular"
        ) {
            if canonical_display(&row.value).map_err(|error| error.to_string())? != row.value {
                return Err(format!(
                    "verb metadata stem is not NFC for {}",
                    row.lexeme_id
                ));
            }
            if detect_script(&row.value) != Script::Cyrillic {
                return Err(format!(
                    "productive verb metadata stem is not Cyrillic for {}",
                    row.lexeme_id
                ));
            }
        }
        if !row.source_feature.starts_with("headword:")
            && !form_values.contains(&(
                row.lexeme_id.as_str(),
                row.source_feature.as_str(),
                row.source_form.as_str(),
            ))
        {
            return Err(format!(
                "verb metadata source cell is absent for {} {}",
                row.lexeme_id, row.source_feature
            ));
        }
        for feature in row
            .crosscheck_features
            .split(" || ")
            .filter(|feature| !feature.is_empty())
        {
            if !form_cells.contains(&(row.lexeme_id.as_str(), feature)) {
                return Err(format!(
                    "verb metadata cross-check is absent for {} {feature}",
                    row.lexeme_id
                ));
            }
        }
        groups
            .entry((
                row.lexeme_id.as_str(),
                row.system.as_str(),
                row.analysis_rank,
            ))
            .or_default()
            .insert(row.field.as_str(), row.value.as_str());
        ranks
            .entry((row.lexeme_id.as_str(), row.system.as_str()))
            .or_default()
            .insert(row.analysis_rank);
    }
    for ((id, system, rank), fields) in groups {
        let required: &[&str] = match system {
            "aspect" => &["aspect"],
            "present" => &["class", "stem"],
            "l-participle" => &["stem"],
            "imperfect" => &["stem", "formation", "variant-policy"],
            "aorist"
            | "imperative"
            | "present-active-participle"
            | "present-passive-participle"
            | "past-active-participle"
            | "past-passive-participle" => &["stem", "formation"],
            _ => return Err(format!("unknown verb metadata system {system} for {id}")),
        };
        for field in required {
            if !fields.contains_key(field) {
                return Err(format!(
                    "incomplete verb metadata analysis {id} {system} {rank}: missing {field}"
                ));
            }
        }
        if system == "present"
            && fields
                .get("class")
                .is_some_and(|class| matches!(*class, "II1" | "II2" | "II3"))
            && !fields.contains_key("first-singular-stem")
        {
            return Err(format!(
                "incomplete verb metadata analysis {id} {system} {rank}: missing first-singular-stem"
            ));
        }
        if system == "aorist" {
            let is_sigmatic = fields.get("formation").is_some_and(|formation| {
                matches!(
                    *formation,
                    "sigmatic-primary" | "sigmatic-secondary" | "sigmatic-vowel"
                )
            });
            if is_sigmatic && !fields.contains_key("second-third-singular") {
                return Err(format!(
                    "incomplete verb metadata analysis {id} {system} {rank}: missing second-third-singular"
                ));
            }
            if !is_sigmatic && fields.contains_key("second-third-singular") {
                return Err(format!(
                    "invalid non-sigmatic aorist metadata analysis {id} {system} {rank}: unexpected second-third-singular"
                ));
            }
        }
        if system == "past-active-participle"
            && fields.get("formation") == Some(&"vush-after-ov-to-u")
            && fields.get("stem").is_none_or(|stem| !stem.ends_with("ов"))
        {
            return Err(format!(
                "invalid ov-to-u verb metadata analysis {id} {system} {rank}"
            ));
        }
    }
    for ((id, system), observed) in ranks {
        for (expected, rank) in observed.into_iter().enumerate() {
            if usize::from(rank) != expected {
                return Err(format!(
                    "non-contiguous verb metadata ranks for {id} {system}: {rank} follows {}",
                    expected.saturating_sub(1)
                ));
            }
        }
    }
    Ok(())
}

fn validate_metadata_code(system: &str, field: &str, value: &str) -> Result<(), String> {
    let valid = match (system, field) {
        ("aspect", "aspect") => matches!(value, "perfective" | "imperfective" | "biaspectual"),
        ("present", "class") => matches!(value, "IA1" | "IA2" | "II1" | "II2" | "II3"),
        ("present", "stem" | "first-singular-stem") => true,
        ("imperfect", "stem")
        | ("aorist", "stem" | "second-third-singular")
        | ("imperative", "stem")
        | ("l-participle", "stem")
        | ("present-active-participle", "stem")
        | ("present-passive-participle", "stem")
        | ("past-active-participle", "stem")
        | ("past-passive-participle", "stem") => true,
        ("imperfect", "formation") => {
            matches!(
                value,
                "a" | "yat-a" | "palatalized-a" | "present-a" | "present-yat-a"
            )
        }
        ("imperfect", "variant-policy") => {
            matches!(
                value,
                "uncontracted-only" | "contracted-only" | "iotated-only"
            )
        }
        ("aorist", "formation") => matches!(
            value,
            "asigmatic" | "new" | "sigmatic-primary" | "sigmatic-secondary" | "sigmatic-vowel"
        ),
        ("imperative", "formation") => matches!(value, "i-series" | "yat-series"),
        ("present-active-participle", "formation") => {
            matches!(
                value,
                "yusht-hard"
                    | "yusht-soft"
                    | "yesht-soft"
                    | "mixed-yusht-soft"
                    | "iotated-yusht-soft"
            )
        }
        ("present-passive-participle", "formation") => {
            matches!(value, "im" | "em" | "iotated-em" | "om")
        }
        ("past-active-participle", "formation") => matches!(
            value,
            "ush" | "ish" | "vush" | "vush-after-j-deletion" | "vush-after-ov-to-u"
        ),
        ("past-passive-participle", "formation") => matches!(value, "t" | "n" | "en"),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(format!(
            "unknown verb metadata code: system={system} field={field} value={value}"
        ))
    }
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
    use crate::ocs::schema::{AliasRow, FormRow, LexemeRow};

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
            verb_metadata: Vec::new(),
            overrides: Vec::new(),
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
