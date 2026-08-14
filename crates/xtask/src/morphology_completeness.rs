use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use synodal_church_slavonic_core::{
    AdjectiveForm, Animacy, FiniteTense, NounDeclension, ParticipleTense, ParticipleVoice,
    VerbSystem,
};

const MATRIX_PATH: &str = "data/morphology/completion.toml";
const REQUIRED_SYSTEM_IDS: &[&str] = &[
    "ocs.noun.declension",
    "ocs.adjective.positive",
    "ocs.adjective.comparison",
    "ocs.pronoun.personal-reflexive",
    "ocs.pronoun.other",
    "ocs.determiner",
    "ocs.numeral",
    "ocs.verb.present",
    "ocs.verb.imperfect.uncontracted",
    "ocs.verb.imperfect.contracted",
    "ocs.verb.imperfect.iotated",
    "ocs.verb.aorist.asigmatic-new",
    "ocs.verb.aorist.sigmatic",
    "ocs.verb.imperative.synthetic",
    "ocs.verb.imperative.analytic",
    "ocs.verb.infinitive-supine-lparticiple",
    "ocs.verb.participle",
    "ocs.verb.irregular-root",
    "ocs.verb.analytic",
    "ocs.verb.verbal-noun",
    "ocs.orthography.canonical",
    "ocs.orthography.accent",
    "ocs.orthography.glagolitic",
    "ocs.lexicon.classification",
    "ocs.invariant-parts-of-speech",
    "syn.noun.declension",
    "syn.noun.irregular",
    "syn.adjective.positive-comparison",
    "syn.adjective.short-superlative",
    "syn.pronoun",
    "syn.determiner",
    "syn.numeral",
    "syn.verb.present",
    "syn.verb.aorist",
    "syn.verb.imperfect",
    "syn.verb.future",
    "syn.verb.past-underspecified",
    "syn.verb.imperative-infinitive-lparticiple",
    "syn.verb.supine",
    "syn.verb.participle",
    "syn.verb.verbal-noun",
    "syn.verb.irregular",
    "syn.analytic.current",
    "syn.analytic.wider",
    "syn.orthography.canonical",
    "syn.orthography.liturgical-accent-abbreviation",
    "syn.lexicon.classification",
    "syn.invariant-parts-of-speech",
    "cross.open-lexicon-provider",
    "cross.provenance-and-prediction",
    "cross.source-frontier",
    "cross.corpus-and-heldout-evaluation",
];
const FINAL_STATES: &[&str] = &[
    "productive-complete",
    "closed-exact-complete",
    "irregular-exact-complete",
    "historically-invalid",
    "absent-from-recension",
    "not-inflectional",
];
const NON_FINAL_STATES: &[&str] = &[
    "unknown",
    "partial",
    "unsupported",
    "source-review-open",
    "implementation-missing",
];

#[derive(Debug, Deserialize)]
struct CompletionMatrix {
    schema_version: u32,
    as_of: String,
    source_frontier: String,
    generated_report: String,
    profiles: Vec<String>,
    system: Vec<System>,
}

#[derive(Debug, Deserialize)]
struct System {
    id: String,
    recension: String,
    category: String,
    part_of_speech: String,
    scope: String,
    profiles: Vec<String>,
    features: Vec<String>,
    subtypes: Vec<String>,
    #[serde(default)]
    verb_systems: Vec<String>,
    realization: String,
    state: String,
    implementation_kind: String,
    rule_ids: Vec<String>,
    sources: Vec<String>,
    citation: String,
    required_metadata: Vec<String>,
    valid_inventory: String,
    invalid_inventory: String,
    implementation: Vec<String>,
    tests: Vec<String>,
    evidence_status: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    gap: String,
}

#[derive(Debug, Deserialize)]
struct SourceFrontier {
    schema_version: u32,
    as_of: String,
    authority_policy: String,
    source: Vec<Source>,
    #[serde(default)]
    discovery_pass: Vec<DiscoveryPass>,
}

#[derive(Debug, Deserialize)]
struct Source {
    id: String,
    title: String,
    author_editor: String,
    edition_revision: String,
    publication_date: String,
    publisher_institution: String,
    stable_url: String,
    retrieved_at: String,
    sha256: String,
    languages: Vec<String>,
    periods: Vec<String>,
    recensions: Vec<String>,
    orthographies: Vec<String>,
    source_type: String,
    epistemic_roles: Vec<String>,
    authority_tier: u8,
    authority_justification: String,
    license: String,
    redistribution: String,
    upstream_lineage: Vec<String>,
    reviewed_sections: Vec<String>,
    rule_ids: Vec<String>,
    impact: String,
    conflicts: Vec<String>,
    access_status: String,
}

#[derive(Debug, Deserialize)]
struct DiscoveryPass {
    id: String,
    performed_at: String,
    languages: Vec<String>,
    queries: Vec<String>,
    catalogs: Vec<String>,
    new_sources: Vec<String>,
    changed_inventory: bool,
    changed_contracts: bool,
    changed_conflicts: bool,
    changed_validation: bool,
    notes: String,
}

struct Audit {
    matrix: CompletionMatrix,
    frontier: SourceFrontier,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => {
                return Err(format!("unknown morphology-completeness argument {value:?}").into());
            }
        }
    }

    let audit = load_and_validate(root)?;
    let report = render(&audit);
    let report_path = root.join(&audit.matrix.generated_report);
    if check {
        if fs::read_to_string(&report_path)? != report {
            return Err(format!(
                "stale {}; rerun cargo xtask morphology-completeness",
                audit.matrix.generated_report
            )
            .into());
        }
        require_complete(&audit)?;
        println!("Church Slavonic morphology completion contract: complete");
    } else {
        fs::write(&report_path, report)?;
        let remaining = audit
            .matrix
            .system
            .iter()
            .filter(|system| !is_final(&system.state))
            .count();
        println!(
            "wrote {} ({remaining} non-final systems remain)",
            audit.matrix.generated_report
        );
    }
    Ok(())
}

/// Keeps the progress inventory and report structurally guarded while the
/// long-running completion goal still contains honest non-final rows.
pub(crate) fn check_progress_artifacts(root: &Path) -> Result<(), Box<dyn Error>> {
    let audit = load_and_validate(root)?;
    let expected = render(&audit);
    if fs::read_to_string(root.join(&audit.matrix.generated_report))? != expected {
        return Err(format!(
            "stale {}; rerun cargo xtask morphology-completeness",
            audit.matrix.generated_report
        )
        .into());
    }
    println!("morphology completion inventory and progress report: current");
    Ok(())
}

fn load_and_validate(root: &Path) -> Result<Audit, Box<dyn Error>> {
    let matrix: CompletionMatrix = toml::from_str(&fs::read_to_string(root.join(MATRIX_PATH))?)?;
    if matrix.schema_version != 1 {
        return Err(format!(
            "{MATRIX_PATH} has unsupported schema version {}",
            matrix.schema_version
        )
        .into());
    }
    if matrix.as_of.is_empty()
        || matrix.generated_report.is_empty()
        || matrix.profiles.is_empty()
        || matrix.system.is_empty()
    {
        return Err(format!("{MATRIX_PATH} has incomplete top-level metadata").into());
    }
    validate_safe_relative_path(&matrix.source_frontier)?;
    validate_safe_relative_path(&matrix.generated_report)?;
    let frontier_path = root.join(&matrix.source_frontier);
    let frontier: SourceFrontier = toml::from_str(&fs::read_to_string(&frontier_path)?)?;
    validate_frontier(&frontier, &frontier_path)?;
    validate_systems(root, &matrix, &frontier)?;
    Ok(Audit { matrix, frontier })
}

fn validate_systems(
    root: &Path,
    matrix: &CompletionMatrix,
    frontier: &SourceFrontier,
) -> Result<(), Box<dyn Error>> {
    let source_ids = frontier
        .source
        .iter()
        .map(|source| source.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    let mut synodal_verb_systems = BTreeSet::new();
    for system in &matrix.system {
        if !ids.insert(system.id.as_str()) {
            return Err(format!("{MATRIX_PATH} has duplicate system ID {:?}", system.id).into());
        }
        validate_system(system, root, &source_ids)?;
        if system.recension == "synodal-russian" {
            synodal_verb_systems.extend(system.verb_systems.iter().map(String::as_str));
        }
    }
    for required in REQUIRED_SYSTEM_IDS {
        if !ids.contains(required) {
            return Err(format!("{MATRIX_PATH} omits required system {required:?}").into());
        }
    }

    let ocs_nouns = matrix
        .system
        .iter()
        .find(|system| system.id == "ocs.noun.declension")
        .ok_or("missing OCS noun system")?;
    let expected_ocs_nouns = BTreeSet::from([
        "o-m-hard",
        "o-n-hard",
        "jo-m-soft",
        "jo-n-soft",
        "a-hard",
        "ja-soft",
        "i-f",
        "i-m",
        "u-m",
        "n-m",
        "n-n",
        "nt-n",
        "r-n",
        "s-n",
        "v-f",
        "indeclinable",
    ]);
    let actual_ocs_nouns = ocs_nouns
        .subtypes
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_ocs_nouns != expected_ocs_nouns {
        return Err(
            "OCS noun completion inventory does not match the core NounClass universe".into(),
        );
    }

    let synodal_nouns = matrix
        .system
        .iter()
        .find(|system| system.id == "syn.noun.declension")
        .ok_or("missing Synodal noun system")?;
    let expected_synodal_nouns = NounDeclension::ALL
        .into_iter()
        .map(|declension| format!("{declension:?}"))
        .collect::<BTreeSet<_>>();
    let actual_synodal_nouns = synodal_nouns
        .subtypes
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_synodal_nouns != expected_synodal_nouns {
        return Err("Synodal noun completion inventory does not match NounDeclension::ALL".into());
    }

    let expected_verb_systems = VerbSystem::ALL
        .into_iter()
        .map(verb_system_code)
        .collect::<BTreeSet<_>>();
    if synodal_verb_systems != expected_verb_systems {
        let missing = expected_verb_systems
            .difference(&synodal_verb_systems)
            .copied()
            .collect::<Vec<_>>();
        let extra = synodal_verb_systems
            .difference(&expected_verb_systems)
            .copied()
            .collect::<Vec<_>>();
        return Err(format!(
            "Synodal completion matrix and VerbSystem::ALL disagree; missing={missing:?}, extra={extra:?}"
        )
        .into());
    }
    Ok(())
}

fn validate_system(
    system: &System,
    root: &Path,
    source_ids: &BTreeSet<&str>,
) -> Result<(), Box<dyn Error>> {
    let scalar_fields = [
        ("recension", system.recension.as_str()),
        ("category", system.category.as_str()),
        ("part_of_speech", system.part_of_speech.as_str()),
        ("scope", system.scope.as_str()),
        ("realization", system.realization.as_str()),
        ("implementation_kind", system.implementation_kind.as_str()),
        ("citation", system.citation.as_str()),
        ("valid_inventory", system.valid_inventory.as_str()),
        ("invalid_inventory", system.invalid_inventory.as_str()),
        ("evidence_status", system.evidence_status.as_str()),
    ];
    if system.id.is_empty()
        || scalar_fields.iter().any(|(_, value)| value.is_empty())
        || system.profiles.is_empty()
        || system.features.is_empty()
        || system.subtypes.is_empty()
        || system.rule_ids.is_empty()
        || system.sources.is_empty()
        || system.required_metadata.is_empty()
        || system.implementation.is_empty()
        || system.tests.is_empty()
    {
        return Err(format!("system {:?} has an incomplete contract", system.id).into());
    }
    if !FINAL_STATES.contains(&system.state.as_str())
        && !NON_FINAL_STATES.contains(&system.state.as_str())
    {
        return Err(format!(
            "system {:?} has unknown completion state {:?}",
            system.id, system.state
        )
        .into());
    }
    if is_final(&system.state) {
        if system.rationale.is_empty() {
            return Err(format!("final system {:?} has no rationale", system.id).into());
        }
        if !system.gap.is_empty() {
            return Err(format!("final system {:?} still declares a gap", system.id).into());
        }
    } else if system.gap.is_empty() {
        return Err(format!("non-final system {:?} has no explicit gap", system.id).into());
    }
    for source in &system.sources {
        if !source_ids.contains(source.as_str()) {
            return Err(format!(
                "system {:?} references unknown frontier source {:?}",
                system.id, source
            )
            .into());
        }
    }
    for reference in system.implementation.iter().chain(&system.tests) {
        validate_repository_reference(root, &system.id, reference)?;
    }
    Ok(())
}

fn validate_frontier(frontier: &SourceFrontier, path: &Path) -> Result<(), Box<dyn Error>> {
    if frontier.schema_version != 1
        || frontier.as_of.is_empty()
        || frontier.authority_policy.is_empty()
        || frontier.source.is_empty()
    {
        return Err(format!("{} has incomplete top-level metadata", path.display()).into());
    }
    let mut ids = BTreeSet::new();
    for source in &frontier.source {
        if !ids.insert(source.id.as_str()) {
            return Err(
                format!("{} has duplicate source ID {:?}", path.display(), source.id).into(),
            );
        }
        let required = [
            source.title.as_str(),
            source.author_editor.as_str(),
            source.edition_revision.as_str(),
            source.publication_date.as_str(),
            source.publisher_institution.as_str(),
            source.stable_url.as_str(),
            source.retrieved_at.as_str(),
            source.source_type.as_str(),
            source.authority_justification.as_str(),
            source.license.as_str(),
            source.redistribution.as_str(),
            source.impact.as_str(),
            source.access_status.as_str(),
        ];
        if source.id.is_empty()
            || required.iter().any(|field| field.is_empty())
            || source.languages.is_empty()
            || source.periods.is_empty()
            || source.recensions.is_empty()
            || source.orthographies.is_empty()
            || source.epistemic_roles.is_empty()
            || source.reviewed_sections.is_empty()
            || source.rule_ids.is_empty()
            || !(1..=6).contains(&source.authority_tier)
        {
            return Err(
                format!("frontier source {:?} has an incomplete contract", source.id).into(),
            );
        }
        if !source.stable_url.starts_with("https://") && !source.stable_url.starts_with("http://") {
            return Err(format!("frontier source {:?} has no stable HTTP URL", source.id).into());
        }
        if !source.sha256.is_empty()
            && (source.sha256.len() != 64
                || !source.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return Err(format!("frontier source {:?} has an invalid SHA-256", source.id).into());
        }
        for lineage in &source.upstream_lineage {
            if lineage.trim().is_empty() {
                return Err(format!("frontier source {:?} has an empty lineage", source.id).into());
            }
        }
        for conflict in &source.conflicts {
            if conflict.trim().is_empty() {
                return Err(
                    format!("frontier source {:?} has an empty conflict", source.id).into(),
                );
            }
        }
    }

    let mut pass_ids = BTreeSet::new();
    for pass in &frontier.discovery_pass {
        if !pass_ids.insert(pass.id.as_str())
            || pass.performed_at.is_empty()
            || pass.languages.is_empty()
            || pass.queries.is_empty()
            || pass.catalogs.is_empty()
            || pass.notes.is_empty()
        {
            return Err(format!("discovery pass {:?} has an incomplete contract", pass.id).into());
        }
        for source in &pass.new_sources {
            if !ids.contains(source.as_str()) {
                return Err(format!(
                    "discovery pass {:?} names unknown source {:?}",
                    pass.id, source
                )
                .into());
            }
        }
    }
    Ok(())
}

fn validate_repository_reference(
    root: &Path,
    system_id: &str,
    reference: &str,
) -> Result<(), Box<dyn Error>> {
    let (relative, marker) = reference
        .split_once('#')
        .map_or((reference, None), |(path, marker)| (path, Some(marker)));
    validate_safe_relative_path(relative)?;
    let path = root.join(relative);
    if !path.is_file() {
        return Err(format!("system {system_id:?} references missing file {relative:?}").into());
    }
    if let Some(marker) = marker {
        if marker.is_empty() || !fs::read_to_string(&path)?.contains(marker) {
            return Err(format!(
                "system {system_id:?} references missing marker {marker:?} in {relative:?}"
            )
            .into());
        }
    }
    Ok(())
}

fn validate_safe_relative_path(value: &str) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!("unsafe repository-relative path {value:?}").into());
    }
    Ok(())
}

fn verb_system_code(system: VerbSystem) -> &'static str {
    match system {
        VerbSystem::Finite(FiniteTense::Present) => "finite:present",
        VerbSystem::Finite(FiniteTense::Future) => "finite:future",
        VerbSystem::Finite(FiniteTense::Past) => "finite:past",
        VerbSystem::Finite(FiniteTense::Imperfect) => "finite:imperfect",
        VerbSystem::Finite(FiniteTense::Aorist) => "finite:aorist",
        VerbSystem::Imperative => "imperative",
        VerbSystem::Infinitive => "infinitive",
        VerbSystem::LParticiple => "l-participle",
        VerbSystem::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        } => "participle:present:active:short",
        VerbSystem::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Long,
        } => "participle:present:active:long",
        VerbSystem::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Short,
        } => "participle:present:passive:short",
        VerbSystem::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Long,
        } => "participle:present:passive:long",
        VerbSystem::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        } => "participle:past:active:short",
        VerbSystem::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Long,
        } => "participle:past:active:long",
        VerbSystem::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Short,
        } => "participle:past:passive:short",
        VerbSystem::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Long,
        } => "participle:past:passive:long",
        VerbSystem::Supine => "supine",
        VerbSystem::VerbalNoun {
            animacy: Animacy::Inanimate,
        } => "verbal-noun:inanimate",
        VerbSystem::VerbalNoun {
            animacy: Animacy::Animate,
        } => "verbal-noun:animate",
    }
}

fn require_complete(audit: &Audit) -> Result<(), Box<dyn Error>> {
    let remaining = audit
        .matrix
        .system
        .iter()
        .filter(|system| !is_final(&system.state))
        .map(|system| format!("{}={}", system.id, system.state))
        .collect::<Vec<_>>();
    if !remaining.is_empty() {
        return Err(format!(
            "morphology completion has {} non-final systems: {}",
            remaining.len(),
            remaining.join(", ")
        )
        .into());
    }
    if !source_discovery_converged(&audit.frontier) {
        return Err(
            "source frontier has not produced two consecutive no-change discovery passes".into(),
        );
    }
    Ok(())
}

fn source_discovery_converged(frontier: &SourceFrontier) -> bool {
    frontier.discovery_pass.len() >= 2
        && frontier.discovery_pass[frontier.discovery_pass.len() - 2..]
            .iter()
            .all(|pass| {
                !pass.changed_inventory
                    && !pass.changed_contracts
                    && !pass.changed_conflicts
                    && !pass.changed_validation
            })
}

fn is_final(state: &str) -> bool {
    FINAL_STATES.contains(&state)
}

fn render(audit: &Audit) -> String {
    let mut by_state = BTreeMap::<&str, usize>::new();
    let mut by_recension = BTreeMap::<&str, (usize, usize)>::new();
    let mut by_category = BTreeMap::<(&str, &str), (usize, usize)>::new();
    for system in &audit.matrix.system {
        *by_state.entry(&system.state).or_default() += 1;
        let recension = by_recension.entry(&system.recension).or_default();
        recension.0 += 1;
        if is_final(&system.state) {
            recension.1 += 1;
        }
        let category = by_category
            .entry((&system.recension, &system.category))
            .or_default();
        category.0 += 1;
        if is_final(&system.state) {
            category.1 += 1;
        }
    }

    let mut output = String::new();
    output.push_str("# Church Slavonic morphology completion progress\n\n");
    output.push_str("This report is generated from `data/morphology/completion.toml` by `cargo xtask morphology-completeness`. It is a progress inventory, not a claim of completion.\n\n");
    output.push_str(&format!(
        "Inventory date: `{}`. Source-frontier date: `{}`.\n\n",
        audit.matrix.as_of, audit.frontier.as_of
    ));
    output.push_str("## Headline\n\n");
    let final_count = audit
        .matrix
        .system
        .iter()
        .filter(|system| is_final(&system.state))
        .count();
    output.push_str(&format!(
        "The matrix contains **{}** required system contracts: **{}** have final states and **{}** remain non-final. Source discovery has {} recorded pass{} and {}.\n\n",
        audit.matrix.system.len(),
        final_count,
        audit.matrix.system.len() - final_count,
        audit.frontier.discovery_pass.len(),
        if audit.frontier.discovery_pass.len() == 1 { "" } else { "es" },
        if source_discovery_converged(&audit.frontier) { "has converged" } else { "has not converged" }
    ));

    output.push_str("## State totals\n\n| State | Systems |\n|---|---:|\n");
    for (state, count) in by_state {
        output.push_str(&format!("| `{state}` | {count} |\n"));
    }
    output.push_str("\n## Recension totals\n\n| Recension | Final | Total |\n|---|---:|---:|\n");
    for (recension, (total, final_rows)) in by_recension {
        output.push_str(&format!("| {recension} | {final_rows} | {total} |\n"));
    }
    output.push_str(
        "\n## Category totals\n\n| Recension | Category | Final | Total |\n|---|---|---:|---:|\n",
    );
    for ((recension, category), (total, final_rows)) in by_category {
        output.push_str(&format!(
            "| {recension} | {category} | {final_rows} | {total} |\n"
        ));
    }

    output.push_str("\n## Non-final systems\n\n");
    for system in audit
        .matrix
        .system
        .iter()
        .filter(|system| !is_final(&system.state))
    {
        output.push_str(&format!(
            "- `{}` — **{}**: {}\n",
            system.id, system.state, system.gap
        ));
    }

    output.push_str("\n## Complete matrix\n\n| ID | Recension | Category | State | Rule IDs | Evidence |\n|---|---|---|---|---|---|\n");
    for system in &audit.matrix.system {
        output.push_str(&format!(
            "| `{}` | {} | {} | `{}` | {} | {} |\n",
            system.id,
            system.recension,
            system.category,
            system.state,
            system
                .rule_ids
                .iter()
                .map(|rule| format!("`{rule}`"))
                .collect::<Vec<_>>()
                .join("<br>"),
            system.evidence_status.replace('|', "\\|")
        ));
    }

    output.push_str("\n## Source frontier\n\n");
    output.push_str(&format!(
        "The frontier contains **{}** source/lineage records. Authority policy: {}\n\n",
        audit.frontier.source.len(),
        audit.frontier.authority_policy
    ));
    output.push_str(
        "| ID | Tier | Type | Recension | Access | Impact |\n|---|---:|---|---|---|---|\n",
    );
    for source in &audit.frontier.source {
        output.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            source.id,
            source.authority_tier,
            source.source_type,
            source.recensions.join(", "),
            source.access_status,
            source.impact.replace('|', "\\|")
        ));
    }

    output.push_str("\n## Discovery passes\n\n");
    for pass in &audit.frontier.discovery_pass {
        output.push_str(&format!(
            "- `{}` ({}) — new sources: {}; changed inventory/contracts/conflicts/validation: `{}/{}/{}/{}`. {}\n",
            pass.id,
            pass.performed_at,
            if pass.new_sources.is_empty() { "none".to_string() } else { pass.new_sources.join(", ") },
            pass.changed_inventory,
            pass.changed_contracts,
            pass.changed_conflicts,
            pass.changed_validation,
            pass.notes
        ));
    }
    output.push_str("\n## Next checkpoint\n\n");
    if let Some(next) = audit
        .matrix
        .system
        .iter()
        .find(|system| !is_final(&system.state))
    {
        output.push_str(&format!("`{}`: {}\n", next.id, next.gap));
    } else if !source_discovery_converged(&audit.frontier) {
        output.push_str("Run and document the remaining source-discovery convergence passes.\n");
    } else {
        output.push_str("Run the full completion audit and final verification gate.\n");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn committed_inventory_is_structurally_complete_and_deterministic() {
        let audit = load_and_validate(&root()).expect("valid completion inventory");
        assert_eq!(audit.matrix.system.len(), REQUIRED_SYSTEM_IDS.len());
        assert_eq!(render(&audit), render(&audit));
        assert!(render(&audit).contains("ocs.verb.aorist.sigmatic"));
        assert!(render(&audit).contains("syn.verb.supine"));
    }

    #[test]
    fn source_frontier_is_complete_and_lineage_aware() {
        let audit = load_and_validate(&root()).expect("valid source frontier");
        assert!(audit.frontier.source.len() >= 14);
        assert!(audit.frontier.source.iter().any(|source| {
            source.id == "love-lmu-ocs-verbs"
                && source
                    .upstream_lineage
                    .iter()
                    .any(|lineage| lineage == "Psalterium Sinaiticum")
        }));
        assert!(!source_discovery_converged(&audit.frontier));
    }

    #[test]
    fn final_check_rejects_honest_non_final_entries() {
        let audit = load_and_validate(&root()).expect("valid completion inventory");
        let error = require_complete(&audit).expect_err("goal is intentionally incomplete");
        assert!(error.to_string().contains("non-final systems"));
    }
}
