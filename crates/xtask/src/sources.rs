use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};

const LOCK_HEADER: &str = "source_id\tartifact_id\ttransport\turl\tpath\tsha256\tsize_bytes\tformat\tsignature\tcontent_types";

#[derive(Clone, Debug, Eq, PartialEq)]
struct LockedArtifact {
    source_id: String,
    artifact_id: String,
    transport: String,
    url: String,
    path: String,
    sha256: String,
    size_bytes: u64,
    format: String,
    signature: String,
    content_types: String,
}

#[derive(Debug, Deserialize)]
struct Inventory {
    #[serde(default)]
    source: Vec<InventorySource>,
}

#[derive(Debug, Deserialize)]
struct InventorySource {
    id: String,
    name: String,
    #[serde(default)]
    download_status: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    List,
    Status,
    Fetch,
    Verify,
    Refresh,
}

#[derive(Debug)]
struct Options {
    action: Action,
    cache: PathBuf,
    source: Option<String>,
    offline: bool,
    accept_new_checksums: bool,
}

#[derive(Debug, Serialize)]
struct RefreshReport {
    schema_version: u8,
    source_id: String,
    changes: Vec<RefreshChange>,
}

#[derive(Debug, Serialize)]
struct RefreshChange {
    artifact_id: String,
    path: String,
    previous_revision: String,
    current_revision: String,
    previous_sha256: String,
    current_sha256: String,
    previous_size_bytes: u64,
    current_size_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LocalStatus {
    Missing,
    Verified,
    WrongSize,
    WrongChecksum,
    InvalidFormat,
}

impl LocalStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Verified => "verified",
            Self::WrongSize => "wrong-size",
            Self::WrongChecksum => "wrong-checksum",
            Self::InvalidFormat => "invalid-format",
        }
    }
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let options = parse_options(args, root)?;
    let lock_path = root.join("references/SOURCE_LOCK.tsv");
    let checksum_path = root.join("references/SHA256SUMS");
    let inventory_path = root.join("references/SOURCES.toml");
    let original_lock = fs::read(&lock_path)?;
    let mut artifacts = parse_lock(&original_lock)?;
    validate_lock(&artifacts)?;
    let inventory = load_inventory(&inventory_path)?;
    validate_inventory_coverage(&inventory, &artifacts)?;
    validate_checksum_alignment(&checksum_path, &artifacts)?;
    validate_wikisource_revision_coverage(
        &root.join("references/WIKISOURCE_REVISIONS.tsv"),
        &artifacts,
    )?;

    match options.action {
        Action::List => list_sources(&inventory, &artifacts, options.source.as_deref()),
        Action::Status => status(&options, &inventory, &artifacts),
        Action::Verify => verify(&options, &artifacts)?,
        Action::Fetch => fetch(&options, &artifacts)?,
        Action::Refresh => refresh(
            &options,
            &mut artifacts,
            &lock_path,
            &checksum_path,
            &root.join("reports/synodal-source-refresh.json"),
        )?,
    }

    if options.action != Action::Refresh && fs::read(&lock_path)? != original_lock {
        return Err("source lock changed during a read-only source operation".into());
    }
    Ok(())
}

pub(crate) fn check_lock(root: &Path) -> Result<(), Box<dyn Error>> {
    let references = root.join("references");
    let artifacts = parse_lock(&fs::read(references.join("SOURCE_LOCK.tsv"))?)?;
    validate_lock(&artifacts)?;
    let inventory = load_inventory(&references.join("SOURCES.toml"))?;
    validate_inventory_coverage(&inventory, &artifacts)?;
    validate_checksum_alignment(&references.join("SHA256SUMS"), &artifacts)?;
    validate_wikisource_revision_coverage(
        &references.join("WIKISOURCE_REVISIONS.tsv"),
        &artifacts,
    )?;
    Ok(())
}

fn parse_options(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<Options, Box<dyn Error>> {
    let action = match args.next().as_deref() {
        Some("list") => Action::List,
        Some("status") => Action::Status,
        Some("fetch") => Action::Fetch,
        Some("verify") => Action::Verify,
        Some("refresh") => Action::Refresh,
        Some(value) => return Err(format!("unknown synodal-sources command {value:?}").into()),
        None => {
            return Err("synodal-sources requires list, status, fetch, verify, or refresh".into());
        }
    };
    let mut cache = root.join("references/downloads");
    let mut source = None;
    let mut offline = false;
    let mut accept_new_checksums = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--cache" => {
                cache = PathBuf::from(args.next().ok_or("--cache requires a path")?);
            }
            "--source" => {
                source = Some(args.next().ok_or("--source requires an ID")?);
            }
            "--offline" => offline = true,
            "--accept-new-checksums" => accept_new_checksums = true,
            value => return Err(format!("unknown synodal-sources argument {value:?}").into()),
        }
    }
    if action == Action::Fetch && offline {
        return Err("fetch cannot be combined with --offline".into());
    }
    if accept_new_checksums && action != Action::Refresh {
        return Err("--accept-new-checksums is valid only with refresh".into());
    }
    if action == Action::Refresh && !accept_new_checksums {
        return Err(
            "refresh requires --accept-new-checksums so upstream drift is explicitly reviewed"
                .into(),
        );
    }
    if action == Action::Refresh && source.is_none() {
        return Err("refresh requires --source SOURCE_ID to bound the reviewed change".into());
    }
    Ok(Options {
        action,
        cache,
        source,
        offline,
        accept_new_checksums,
    })
}

fn load_inventory(path: &Path) -> Result<Inventory, Box<dyn Error>> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

fn parse_lock(bytes: &[u8]) -> Result<Vec<LockedArtifact>, Box<dyn Error>> {
    let text = std::str::from_utf8(bytes)?;
    let mut lines = text.lines();
    if lines.next() != Some(LOCK_HEADER) {
        return Err("invalid source-lock header".into());
    }
    let mut artifacts = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 10 {
            return Err(format!(
                "invalid source-lock row {}: expected 10 columns, found {}",
                offset + 2,
                fields.len()
            )
            .into());
        }
        artifacts.push(LockedArtifact {
            source_id: fields[0].into(),
            artifact_id: fields[1].into(),
            transport: fields[2].into(),
            url: fields[3].into(),
            path: fields[4].into(),
            sha256: fields[5].into(),
            size_bytes: fields[6]
                .parse()
                .map_err(|_| format!("invalid size in source-lock row {}", offset + 2))?,
            format: fields[7].into(),
            signature: fields[8].into(),
            content_types: fields[9].into(),
        });
    }
    if artifacts.is_empty() {
        return Err("source lock contains no artifacts".into());
    }
    Ok(artifacts)
}

fn validate_lock(artifacts: &[LockedArtifact]) -> Result<(), Box<dyn Error>> {
    let mut artifact_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for artifact in artifacts {
        if artifact.source_id.is_empty()
            || artifact.artifact_id.is_empty()
            || artifact.url.is_empty()
            || artifact.size_bytes == 0
        {
            return Err(format!("incomplete source lock artifact {}", artifact.artifact_id).into());
        }
        if artifact.sha256.len() != 64
            || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!("invalid SHA-256 for {}", artifact.artifact_id).into());
        }
        if !matches!(
            artifact.transport.as_str(),
            "direct" | "fup" | "wikisource-titles" | "wikisource-export" | "mediawiki-revision"
        ) {
            return Err(format!(
                "unknown transport {:?} for {}",
                artifact.transport, artifact.artifact_id
            )
            .into());
        }
        validate_relative_download_path(&artifact.path)?;
        if !artifact_ids.insert((artifact.source_id.clone(), artifact.artifact_id.clone())) {
            return Err(format!("duplicate artifact ID {}", artifact.artifact_id).into());
        }
        if !paths.insert(artifact.path.clone()) {
            return Err(format!("duplicate artifact path {}", artifact.path).into());
        }
    }
    Ok(())
}

fn validate_relative_download_path(path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(path);
    let mut components = path.components();
    if components.next() != Some(Component::Normal("downloads".as_ref()))
        || components.any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe source-lock path {}", path.display()).into());
    }
    Ok(())
}

fn validate_inventory_coverage(
    inventory: &Inventory,
    artifacts: &[LockedArtifact],
) -> Result<(), Box<dyn Error>> {
    let artifact_sources: BTreeSet<&str> = artifacts
        .iter()
        .map(|artifact| artifact.source_id.as_str())
        .collect();
    let inventory_sources: BTreeSet<&str> = inventory
        .source
        .iter()
        .map(|source| source.id.as_str())
        .collect();
    for source in &inventory.source {
        if source.download_status.is_none() && !artifact_sources.contains(source.id.as_str()) {
            return Err(format!("downloadable source {} has no locked artifact", source.id).into());
        }
    }
    for source_id in artifact_sources {
        if !inventory_sources.contains(source_id) {
            return Err(format!("locked artifact has unknown source ID {source_id}").into());
        }
    }
    Ok(())
}

fn validate_checksum_alignment(
    checksum_path: &Path,
    artifacts: &[LockedArtifact],
) -> Result<(), Box<dyn Error>> {
    let mut expected = BTreeMap::new();
    for artifact in artifacts {
        expected.insert(artifact.path.as_str(), artifact.sha256.as_str());
    }
    let text = fs::read_to_string(checksum_path)?;
    let mut actual = BTreeMap::new();
    for (offset, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let Some((checksum, path)) = line.split_once("  ") else {
            return Err(format!("invalid SHA256SUMS row {}", offset + 1).into());
        };
        if actual.insert(path, checksum).is_some() {
            return Err(format!("duplicate SHA256SUMS path {path}").into());
        }
    }
    if actual != expected {
        return Err("SOURCE_LOCK.tsv and SHA256SUMS disagree".into());
    }
    Ok(())
}

fn validate_wikisource_revision_coverage(
    revision_path: &Path,
    artifacts: &[LockedArtifact],
) -> Result<(), Box<dyn Error>> {
    const HEADER: &str = "title\tpage_id\trevision_id\ttimestamp\tmediawiki_sha1";
    if !artifacts
        .iter()
        .any(|artifact| artifact.source_id == "wikisource-church-slavonic-bible-2026-08-09")
    {
        return Ok(());
    }
    let text = fs::read_to_string(revision_path)?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err("invalid WIKISOURCE_REVISIONS.tsv header".into());
    }
    let mut revisions = BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5
            || fields[0].is_empty()
            || fields[1].parse::<u64>().is_err()
            || fields[2].parse::<u64>().is_err()
            || fields[3].is_empty()
            || fields[4].is_empty()
        {
            return Err(format!("invalid Wikisource revision row {}", offset + 2).into());
        }
        if !revisions.insert(fields[2].to_owned()) {
            return Err(format!("duplicate Wikisource revision {}", fields[2]).into());
        }
    }
    let mut locked = BTreeSet::new();
    for artifact in artifacts
        .iter()
        .filter(|artifact| artifact.source_id == "wikisource-church-slavonic-bible-2026-08-09")
    {
        if artifact.transport != "mediawiki-revision" {
            return Err(format!(
                "Wikisource artifact {} is not revision-pinned",
                artifact.artifact_id
            )
            .into());
        }
        let revision = artifact
            .url
            .split("revision_id=")
            .nth(1)
            .ok_or("Wikisource artifact URL omits revision_id")?;
        if !locked.insert(revision.to_owned()) {
            return Err(format!("duplicate locked Wikisource revision {revision}").into());
        }
        if !artifact.path.ends_with(&format!("/{revision}.wikitext")) {
            return Err(
                format!("Wikisource revision {revision} has a mismatched artifact path").into(),
            );
        }
    }
    if revisions != locked {
        return Err("Wikisource revision manifest and source lock disagree".into());
    }
    Ok(())
}

fn list_sources(inventory: &Inventory, artifacts: &[LockedArtifact], selected: Option<&str>) {
    let mut counts = BTreeMap::<&str, usize>::new();
    for artifact in artifacts {
        *counts.entry(&artifact.source_id).or_default() += 1;
    }
    for source in &inventory.source {
        if selected.is_some_and(|selected| selected != source.id) {
            continue;
        }
        let artifact_count = counts.get(source.id.as_str()).copied().unwrap_or_default();
        if artifact_count > 0 {
            println!(
                "{}\t{} artifacts\t{}",
                source.id, artifact_count, source.name
            );
            if let Some(reason) = &source.download_status {
                println!("{}\tpartial-source\t{}", source.id, reason);
            }
        } else if let Some(reason) = &source.download_status {
            println!("{}\tmetadata-only\t{}\t{}", source.id, source.name, reason);
        }
    }
}

fn status(options: &Options, inventory: &Inventory, artifacts: &[LockedArtifact]) {
    let selected = selected_artifacts(options, artifacts);
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for artifact in selected {
        let path = artifact_cache_path(&options.cache, artifact);
        let local_status = match inspect_artifact(&path, artifact) {
            Ok(status) => status,
            Err(error) => {
                eprintln!(
                    "failed to inspect {} at {}: {error}",
                    artifact.artifact_id,
                    path.display()
                );
                LocalStatus::InvalidFormat
            }
        };
        *counts.entry(local_status.label()).or_default() += 1;
        println!(
            "{}\t{}\t{}\t{}",
            local_status.label(),
            artifact.source_id,
            artifact.artifact_id,
            path.display()
        );
    }
    for source in &inventory.source {
        if options
            .source
            .as_deref()
            .is_some_and(|selected| selected != source.id)
        {
            continue;
        }
        if let Some(reason) = &source.download_status {
            let has_artifacts = artifacts
                .iter()
                .any(|artifact| artifact.source_id == source.id);
            let label = if has_artifacts {
                "partial-source"
            } else {
                "metadata-only"
            };
            println!("{label}\t{}\t{}", source.id, reason);
        }
    }
    eprintln!(
        "source status: {} verified, {} missing, {} invalid",
        counts.get("verified").copied().unwrap_or_default(),
        counts.get("missing").copied().unwrap_or_default(),
        counts
            .iter()
            .filter(|(name, _)| **name != "verified" && **name != "missing")
            .map(|(_, count)| count)
            .sum::<usize>()
    );
}

fn verify(options: &Options, artifacts: &[LockedArtifact]) -> Result<(), Box<dyn Error>> {
    if options.offline {
        println!("offline verification: no network requests will be made");
    }
    let selected = selected_artifacts(options, artifacts);
    ensure_selection(options, &selected)?;
    for artifact in &selected {
        let path = artifact_cache_path(&options.cache, artifact);
        match inspect_artifact(&path, artifact)? {
            LocalStatus::Verified => {
                println!("verified  {}  {}", artifact.source_id, artifact.path)
            }
            status => {
                return Err(format!(
                    "{} is {}: {}",
                    artifact.artifact_id,
                    status.label(),
                    path.display()
                )
                .into());
            }
        }
    }
    println!("verified {} locked source artifacts", selected.len());
    Ok(())
}

fn fetch(options: &Options, artifacts: &[LockedArtifact]) -> Result<(), Box<dyn Error>> {
    require_curl()?;
    let selected = selected_artifacts(options, artifacts);
    ensure_selection(options, &selected)?;
    let missing_mediawiki: Vec<&LockedArtifact> = selected
        .iter()
        .copied()
        .filter(|artifact| {
            artifact.transport == "mediawiki-revision"
                && inspect_artifact(&artifact_cache_path(&options.cache, artifact), artifact)
                    .is_ok_and(|status| status == LocalStatus::Missing)
        })
        .collect();
    if !missing_mediawiki.is_empty() {
        fetch_mediawiki_revision_batches(&options.cache, &missing_mediawiki)?;
    }
    for artifact in &selected {
        let destination = artifact_cache_path(&options.cache, artifact);
        match inspect_artifact(&destination, artifact)? {
            LocalStatus::Verified => {
                println!("present   {}", artifact.path);
                continue;
            }
            LocalStatus::Missing => {}
            status => {
                return Err(format!(
                    "refusing to replace {} cached artifact {} without explicit refresh",
                    status.label(),
                    destination.display()
                )
                .into());
            }
        }
        download_locked_artifact(artifact, &options.cache, &destination)?;
        if inspect_artifact(&destination, artifact)? != LocalStatus::Verified {
            return Err(format!(
                "downloaded artifact failed lock verification: {}",
                artifact.path
            )
            .into());
        }
        println!("fetched   {}", artifact.path);
    }
    println!(
        "fetched or verified {} locked source artifacts",
        selected.len()
    );
    Ok(())
}

fn fetch_mediawiki_revision_batches(
    cache: &Path,
    artifacts: &[&LockedArtifact],
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(cache)?;
    for (chunk_index, chunk) in artifacts.chunks(50).enumerate() {
        let mut by_revision = BTreeMap::new();
        for artifact in chunk {
            let revision_id = mediawiki_revision_id(artifact)?;
            if by_revision.insert(revision_id, *artifact).is_some() {
                return Err(format!("duplicate MediaWiki revision {revision_id}").into());
            }
        }
        let revision_ids = by_revision.keys().copied().collect::<Vec<_>>();
        let response = cache.join(format!(
            ".mediawiki-revisions-{}-{chunk_index}.json",
            std::process::id()
        ));
        let endpoint = mediawiki_endpoint(chunk[0])?;
        for artifact in chunk {
            if mediawiki_endpoint(artifact)? != endpoint {
                return Err("a MediaWiki batch cannot mix API endpoints".into());
            }
        }
        let status = Command::new("curl")
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--get",
                "--retry",
                "10",
                "--retry-all-errors",
                "--retry-delay",
                "5",
                "--user-agent",
                "church-slavonic-source-fetcher/2.0 (revision-pinned research corpus)",
                endpoint,
                "--data-urlencode",
                "action=query",
                "--data-urlencode",
                "format=json",
                "--data-urlencode",
                "formatversion=2",
                "--data-urlencode",
                "prop=revisions",
                "--data-urlencode",
            ])
            .arg(format!(
                "revids={}",
                revision_ids
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join("|")
            ))
            .args([
                "--data-urlencode",
                "rvprop=ids|timestamp|sha1|content",
                "--data-urlencode",
                "rvslots=main",
                "--output",
            ])
            .arg(&response)
            .status()?;
        if !status.success() {
            return Err(format!("MediaWiki revision batch {chunk_index} failed").into());
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&response)?)?;
        let pages = value
            .get("query")
            .and_then(|query| query.get("pages"))
            .and_then(serde_json::Value::as_array)
            .ok_or("MediaWiki batch response contains no pages")?;
        let mut received = BTreeSet::new();
        for page in pages {
            let revisions = page
                .get("revisions")
                .and_then(serde_json::Value::as_array)
                .ok_or("MediaWiki page contains no revisions")?;
            for revision in revisions {
                let revision_id = revision
                    .get("revid")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or("MediaWiki revision has no numeric ID")?;
                let artifact = by_revision.get(&revision_id).ok_or_else(|| {
                    format!("MediaWiki returned unexpected revision {revision_id}")
                })?;
                let content = revision
                    .get("slots")
                    .and_then(|slots| slots.get("main"))
                    .and_then(|slot| slot.get("content"))
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        format!("MediaWiki revision {revision_id} has no main-slot content")
                    })?;
                admit_bytes(cache, artifact, content.as_bytes())?;
                received.insert(revision_id);
            }
        }
        let expected: BTreeSet<u64> = by_revision.keys().copied().collect();
        if received != expected {
            return Err(format!(
                "MediaWiki batch omitted revisions: {:?}",
                expected.difference(&received).collect::<Vec<_>>()
            )
            .into());
        }
        fs::remove_file(response)?;
    }
    Ok(())
}

fn admit_bytes(
    cache: &Path,
    artifact: &LockedArtifact,
    bytes: &[u8],
) -> Result<(), Box<dyn Error>> {
    let destination = artifact_cache_path(cache, artifact);
    let partial = destination.with_extension(format!(
        "{}.partial",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("artifact")
    ));
    if let Some(parent) = partial.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&partial, bytes)?;
    let actual_size = fs::metadata(&partial)?.len();
    let actual_sha256 = sha256_file(&partial)?;
    if actual_size != artifact.size_bytes || actual_sha256 != artifact.sha256 {
        let rejected = partial.with_extension("unexpected");
        if rejected.exists() {
            fs::remove_file(&rejected)?;
        }
        fs::rename(&partial, &rejected)?;
        return Err(format!(
            "upstream drift for {}: expected {} bytes {}, received {} bytes {}; retained at {}",
            artifact.artifact_id,
            artifact.size_bytes,
            artifact.sha256,
            actual_size,
            actual_sha256,
            rejected.display()
        )
        .into());
    }
    validate_signature(&partial, &artifact.signature, &artifact.format)?;
    fs::rename(partial, destination)?;
    Ok(())
}

fn mediawiki_revision_id(artifact: &LockedArtifact) -> Result<u64, Box<dyn Error>> {
    let (_, query) = artifact
        .url
        .split_once('?')
        .ok_or("mediawiki-revision URL requires revision_id query data")?;
    let revision_id = query
        .split('&')
        .find_map(|field| field.strip_prefix("revision_id="))
        .ok_or("mediawiki-revision URL has no revision_id")?;
    Ok(revision_id.parse()?)
}

fn mediawiki_endpoint(artifact: &LockedArtifact) -> Result<&str, Box<dyn Error>> {
    artifact
        .url
        .split_once('?')
        .map(|(endpoint, _)| endpoint)
        .ok_or_else(|| "mediawiki-revision URL requires revision_id query data".into())
}

fn refresh(
    options: &Options,
    artifacts: &mut [LockedArtifact],
    lock_path: &Path,
    checksum_path: &Path,
    report_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if !options.accept_new_checksums {
        return Err("refresh was not explicitly authorized".into());
    }
    require_curl()?;
    let selected_indices: Vec<usize> = artifacts
        .iter()
        .enumerate()
        .filter(|(_, artifact)| {
            options
                .source
                .as_deref()
                .is_none_or(|source| source == artifact.source_id)
        })
        .map(|(index, _)| index)
        .collect();
    if selected_indices.is_empty() {
        return Err("refresh selected no locked artifacts".into());
    }

    let staging = options
        .cache
        .join(format!(".refresh-staging-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)?;
    let mut changes = Vec::new();
    for &index in &selected_indices {
        let artifact = &artifacts[index];
        let refresh_path = artifact_cache_path(&staging, artifact);
        if let Some(parent) = refresh_path.parent() {
            fs::create_dir_all(parent)?;
        }
        download_unlocked_artifact(artifact, &staging, &refresh_path, false)?;
        validate_signature(&refresh_path, &artifact.signature, &artifact.format)?;
        validate_container(&refresh_path, &artifact.format)?;
        let new_size = fs::metadata(&refresh_path)?.len();
        let new_sha256 = sha256_file(&refresh_path)?;
        println!(
            "refresh {}: {} {} -> {} {}",
            artifact.artifact_id, artifact.sha256, artifact.size_bytes, new_sha256, new_size
        );
        changes.push(RefreshChange {
            artifact_id: artifact.artifact_id.clone(),
            path: artifact.path.clone(),
            // The exact mutable-source revision is encoded in the locked URL. A
            // URL change is reviewed in the same report as a byte change.
            previous_revision: artifact.url.clone(),
            current_revision: artifact.url.clone(),
            previous_sha256: artifact.sha256.clone(),
            current_sha256: new_sha256.clone(),
            previous_size_bytes: artifact.size_bytes,
            current_size_bytes: new_size,
        });
        artifacts[index].sha256 = new_sha256;
        artifacts[index].size_bytes = new_size;
    }

    let report = serde_json::to_string_pretty(&RefreshReport {
        schema_version: 1,
        source_id: options.source.clone().ok_or("refresh requires a source")?,
        changes,
    })? + "\n";
    let lock_staged = staging.join("SOURCE_LOCK.tsv");
    let checksums_staged = staging.join("SHA256SUMS");
    let report_staged = staging.join("synodal-source-refresh.json");
    fs::write(&lock_staged, serialize_lock(artifacts))?;
    fs::write(&checksums_staged, serialize_checksums(artifacts))?;
    fs::write(&report_staged, report)?;

    // No cache or manifest path is touched until every refreshed byte has been
    // downloaded, structurally validated, hashed, and its report staged.
    for &index in &selected_indices {
        let artifact = &artifacts[index];
        let staged_artifact = artifact_cache_path(&staging, artifact);
        let destination = artifact_cache_path(&options.cache, artifact);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        replace_file(&staged_artifact, &destination)?;
    }
    atomic_write(lock_path, &fs::read(lock_staged)?)?;
    atomic_write(checksum_path, &fs::read(checksums_staged)?)?;
    atomic_write(report_path, &fs::read(report_staged)?)?;
    fs::remove_dir_all(staging)?;
    println!("updated {}", lock_path.display());
    println!("wrote review report {}", report_path.display());
    Ok(())
}

fn selected_artifacts<'a>(
    options: &Options,
    artifacts: &'a [LockedArtifact],
) -> Vec<&'a LockedArtifact> {
    artifacts
        .iter()
        .filter(|artifact| {
            options
                .source
                .as_deref()
                .is_none_or(|source| source == artifact.source_id)
        })
        .collect()
}

fn ensure_selection(options: &Options, selected: &[&LockedArtifact]) -> Result<(), Box<dyn Error>> {
    if selected.is_empty() {
        if let Some(source) = &options.source {
            Err(format!("source {source:?} has no downloadable locked artifacts").into())
        } else {
            Err("source command selected no artifacts".into())
        }
    } else {
        Ok(())
    }
}

fn artifact_cache_path(cache: &Path, artifact: &LockedArtifact) -> PathBuf {
    let relative = Path::new(&artifact.path)
        .strip_prefix("downloads")
        .unwrap_or_else(|_| Path::new(&artifact.path));
    cache.join(relative)
}

fn inspect_artifact(path: &Path, artifact: &LockedArtifact) -> Result<LocalStatus, Box<dyn Error>> {
    if !path.exists() {
        return Ok(LocalStatus::Missing);
    }
    if !path.is_file() || fs::metadata(path)?.len() != artifact.size_bytes {
        return Ok(LocalStatus::WrongSize);
    }
    if sha256_file(path)? != artifact.sha256 {
        return Ok(LocalStatus::WrongChecksum);
    }
    if validate_signature(path, &artifact.signature, &artifact.format).is_err() {
        return Ok(LocalStatus::InvalidFormat);
    }
    Ok(LocalStatus::Verified)
}

fn download_locked_artifact(
    artifact: &LockedArtifact,
    cache: &Path,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    let partial = destination.with_extension(format!(
        "{}.partial",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("artifact")
    ));
    if let Some(parent) = partial.parent() {
        fs::create_dir_all(parent)?;
    }
    download_unlocked_artifact(artifact, cache, &partial, true)?;
    let actual_size = fs::metadata(&partial)?.len();
    let actual_sha256 = sha256_file(&partial)?;
    if actual_size != artifact.size_bytes || actual_sha256 != artifact.sha256 {
        let rejected = partial.with_extension("unexpected");
        if rejected.exists() {
            fs::remove_file(&rejected)?;
        }
        fs::rename(&partial, &rejected)?;
        return Err(format!(
            "upstream drift for {}: expected {} bytes {}, received {} bytes {}; retained at {}",
            artifact.artifact_id,
            artifact.size_bytes,
            artifact.sha256,
            actual_size,
            actual_sha256,
            rejected.display()
        )
        .into());
    }
    validate_signature(&partial, &artifact.signature, &artifact.format)?;
    validate_container(&partial, &artifact.format)?;
    fs::rename(partial, destination)?;
    Ok(())
}

fn download_unlocked_artifact(
    artifact: &LockedArtifact,
    cache: &Path,
    destination: &Path,
    resume: bool,
) -> Result<(), Box<dyn Error>> {
    match artifact.transport.as_str() {
        "direct" | "fup" => curl_download(artifact, destination, resume),
        "wikisource-titles" => download_wikisource_titles(artifact, destination),
        "wikisource-export" => download_wikisource_export(artifact, cache, destination),
        "mediawiki-revision" => download_mediawiki_revision(artifact, destination),
        transport => Err(format!("unsupported source transport {transport}").into()),
    }
}

fn curl_download(
    artifact: &LockedArtifact,
    destination: &Path,
    resume: bool,
) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("curl");
    command.args([
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--retry",
        "4",
        "--retry-delay",
        "2",
    ]);
    if resume && destination.exists() {
        command.args(["--continue-at", "-"]);
    }
    if artifact.transport == "fup" {
        command.args([
            "--http1.1",
            "--user-agent",
            "Mozilla/5.0 source-lock-fetcher/2.0",
            "--referer",
            "https://books.fupress.com/catalogue/old-church-slavic/8465",
            "--header",
            "Accept: application/xml,text/xml,application/pdf,*/*;q=0.8",
        ]);
    } else {
        command.args(["--user-agent", "church-slavonic-source-fetcher/2.0"]);
    }
    let header_path = destination.with_extension("headers.tmp");
    let status = command
        .arg("--dump-header")
        .arg(&header_path)
        .arg("--output")
        .arg(destination)
        .arg(&artifact.url)
        .status()?;
    if status.success() {
        if artifact.content_types != "-" {
            validate_content_type(&header_path, &artifact.content_types)?;
        }
        if header_path.exists() {
            fs::remove_file(header_path)?;
        }
        Ok(())
    } else {
        let _ = fs::remove_file(header_path);
        Err(format!("curl failed for {} with {status}", artifact.url).into())
    }
}

fn download_wikisource_titles(
    artifact: &LockedArtifact,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    let response = destination.with_extension("api.json");
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--get",
            "--retry",
            "8",
            "--retry-all-errors",
            "--retry-delay",
            "3",
            &artifact.url,
            "--data-urlencode",
            "action=query",
            "--data-urlencode",
            "format=json",
            "--data-urlencode",
            "formatversion=2",
            "--data-urlencode",
            "prop=links",
            "--data-urlencode",
            "titles=Бі́блїа",
            "--data-urlencode",
            "plnamespace=0",
            "--data-urlencode",
            "pllimit=max",
            "--output",
        ])
        .arg(&response)
        .status()?;
    if !status.success() {
        return Err("Wikisource title discovery failed".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&response)?)?;
    let mut titles = BTreeSet::from(["Бі́блїа".to_owned()]);
    if let Some(pages) = value
        .get("query")
        .and_then(|query| query.get("pages"))
        .and_then(serde_json::Value::as_array)
    {
        for page in pages {
            if let Some(links) = page.get("links").and_then(serde_json::Value::as_array) {
                for link in links {
                    if let Some(title) = link.get("title").and_then(serde_json::Value::as_str) {
                        titles.insert(title.to_owned());
                    }
                }
            }
        }
    }
    let mut output = String::new();
    for title in titles {
        output.push_str(&title);
        output.push('\n');
    }
    fs::write(destination, output)?;
    fs::remove_file(response)?;
    Ok(())
}

fn download_wikisource_export(
    artifact: &LockedArtifact,
    cache: &Path,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    let titles = cache.join("wikisource-church-slavonic-bible/titles.txt");
    if !titles.is_file() {
        return Err(format!(
            "Wikisource export requires locked title inventory {}",
            titles.display()
        )
        .into());
    }
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "8",
            "--retry-all-errors",
            "--retry-delay",
            "3",
            "--request",
            "POST",
            &artifact.url,
            "--data-urlencode",
        ])
        .arg(format!("pages@{}", titles.display()))
        .args(["--data", "curonly=1", "--data", "action=submit", "--output"])
        .arg(destination)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err("Wikisource export failed".into())
    }
}

fn download_mediawiki_revision(
    artifact: &LockedArtifact,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    let (endpoint, query) = artifact
        .url
        .split_once('?')
        .ok_or("mediawiki-revision URL requires revision_id query data")?;
    let revision_id = query
        .split('&')
        .find_map(|field| field.strip_prefix("revision_id="))
        .ok_or("mediawiki-revision URL has no revision_id")?;
    if revision_id.is_empty() || !revision_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid MediaWiki revision ID {revision_id:?}").into());
    }
    let response = destination.with_extension("api.json");
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--get",
            "--retry",
            "8",
            "--retry-all-errors",
            "--retry-delay",
            "3",
            endpoint,
            "--data-urlencode",
            "action=query",
            "--data-urlencode",
            "format=json",
            "--data-urlencode",
            "formatversion=2",
            "--data-urlencode",
            "prop=revisions",
            "--data-urlencode",
        ])
        .arg(format!("revids={revision_id}"))
        .args([
            "--data-urlencode",
            "rvprop=ids|timestamp|sha1|content",
            "--data-urlencode",
            "rvslots=main",
            "--output",
        ])
        .arg(&response)
        .status()?;
    if !status.success() {
        return Err(format!("MediaWiki revision request {revision_id} failed").into());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&response)?)?;
    let revision = value
        .get("query")
        .and_then(|query| query.get("pages"))
        .and_then(serde_json::Value::as_array)
        .and_then(|pages| pages.first())
        .and_then(|page| page.get("revisions"))
        .and_then(serde_json::Value::as_array)
        .and_then(|revisions| revisions.first())
        .ok_or_else(|| format!("MediaWiki returned no revision {revision_id}"))?;
    if revision.get("revid").and_then(serde_json::Value::as_u64)
        != Some(revision_id.parse::<u64>()?)
    {
        return Err(format!("MediaWiki returned the wrong revision for {revision_id}").into());
    }
    let content = revision
        .get("slots")
        .and_then(|slots| slots.get("main"))
        .and_then(|slot| slot.get("content"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("MediaWiki revision {revision_id} has no main-slot content"))?;
    fs::write(destination, content.as_bytes())?;
    fs::remove_file(response)?;
    Ok(())
}

fn validate_signature(path: &Path, signature: &str, format: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut prefix = vec![0_u8; 8192];
    let count = file.read(&mut prefix)?;
    prefix.truncate(count);
    if prefix.is_empty() {
        return Err(format!("empty artifact {}", path.display()).into());
    }
    let prefix_without_bom = prefix.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&prefix);
    let trimmed = prefix_without_bom
        .iter()
        .copied()
        .skip_while(|byte| byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    let looks_html = contains_ascii_case_insensitive(&trimmed, b"<html")
        || contains_ascii_case_insensitive(&trimmed, b"<!doctype html");
    let valid = match signature {
        "pdf" => prefix.starts_with(b"%PDF-") && !looks_html,
        "zip" => prefix.starts_with(b"PK\x03\x04") && !looks_html,
        "gzip" => prefix.starts_with(&[0x1f, 0x8b]) && !looks_html,
        "bzip2" => prefix.starts_with(b"BZh") && !looks_html,
        "djvu" => prefix.starts_with(b"AT&TFORM") && !looks_html,
        "xml" => {
            !looks_html
                && (trimmed.starts_with(b"<?xml")
                    || trimmed.starts_with(b"<mediawiki")
                    || trimmed.starts_with(b"<TEI"))
        }
        "html" => looks_html,
        "json" => trimmed.starts_with(b"{") || trimmed.starts_with(b"["),
        "text" => !prefix.contains(&0) && !looks_html,
        "binary" => !looks_html,
        value => return Err(format!("unknown signature class {value:?}").into()),
    };
    if valid {
        Ok(())
    } else {
        Err(format!(
            "{} does not match expected {format}/{signature} content",
            path.display()
        )
        .into())
    }
}

fn validate_content_type(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    let headers = fs::read_to_string(path)?;
    let actual = headers
        .lines()
        .rev()
        .find_map(|line| {
            line.split_once(':').and_then(|(name, value)| {
                name.eq_ignore_ascii_case("content-type").then(|| {
                    value
                        .trim()
                        .split(';')
                        .next()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                })
            })
        })
        .ok_or("HTTP response omitted Content-Type")?;
    if expected
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate.eq_ignore_ascii_case(&actual))
    {
        Ok(())
    } else {
        Err(format!("unexpected HTTP Content-Type {actual:?}; expected {expected}").into())
    }
}

fn validate_container(path: &Path, format: &str) -> Result<(), Box<dyn Error>> {
    let (command, arguments): (&str, &[&str]) = match format {
        "tar.gz" => ("tar", &["-tzf"]),
        "zip" => ("unzip", &["-tqq"]),
        "xml.bz2" => ("bzip2", &["-t"]),
        "jsonl.gz" => ("gzip", &["-t"]),
        _ => return Ok(()),
    };
    require_command(
        command,
        &format!("install {command} to validate downloaded {format} artifacts"),
    )?;
    let status = Command::new(command).args(arguments).arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{format} integrity validation failed for {}",
            path.display()
        )
        .into())
    }
}

fn contains_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| {
        window
            .iter()
            .zip(needle)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
    })
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn require_command(command: &str, remediation: &str) -> Result<(), Box<dyn Error>> {
    // Info-ZIP treats `--version` as the combined short flags `-n -o` and
    // exits 10 even when the executable is healthy. Use its documented `-v`
    // probe; the other archive tools accept GNU-style `--version`.
    match Command::new(command)
        .arg(command_version_argument(command))
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(format!("required command {command:?} is unavailable; {remediation}").into()),
    }
}

fn command_version_argument(command: &str) -> &'static str {
    if command == "unzip" {
        "-v"
    } else {
        "--version"
    }
}

fn require_curl() -> Result<(), Box<dyn Error>> {
    let output = Command::new("curl")
        .arg("--version")
        .output()
        .map_err(|_| "required command \"curl\" is unavailable; install curl 7.71 or newer")?;
    if !output.status.success() {
        return Err("curl --version failed; install curl 7.71 or newer".into());
    }
    let version = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .nth(1)
        .ok_or("could not parse curl version")?
        .split('.')
        .take(2)
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()?;
    if version.as_slice() < &[7, 71] {
        return Err("curl 7.71 or newer is required for safe retry and resume behavior".into());
    }
    Ok(())
}

fn replace_file(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    let backup = destination.with_extension(format!(
        "{}.refresh-backup",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("artifact")
    ));
    if backup.exists() {
        fs::remove_file(&backup)?;
    }
    if destination.exists() {
        fs::rename(destination, &backup)?;
    }
    match fs::rename(source, destination) {
        Ok(()) => {
            if backup.exists() {
                fs::remove_file(backup)?;
            }
            Ok(())
        }
        Err(error) => {
            if backup.exists() {
                fs::rename(backup, destination)?;
            }
            Err(error.into())
        }
    }
}

fn serialize_lock(artifacts: &[LockedArtifact]) -> String {
    let mut output = String::from(LOCK_HEADER);
    output.push('\n');
    for artifact in artifacts {
        output.push_str(
            &[
                artifact.source_id.as_str(),
                artifact.artifact_id.as_str(),
                artifact.transport.as_str(),
                artifact.url.as_str(),
                artifact.path.as_str(),
                artifact.sha256.as_str(),
                &artifact.size_bytes.to_string(),
                artifact.format.as_str(),
                artifact.signature.as_str(),
                artifact.content_types.as_str(),
            ]
            .join("\t"),
        );
        output.push('\n');
    }
    output
}

fn serialize_checksums(artifacts: &[LockedArtifact]) -> String {
    let mut rows: Vec<(&str, &str)> = artifacts
        .iter()
        .map(|artifact| (artifact.path.as_str(), artifact.sha256.as_str()))
        .collect();
    rows.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let mut output = String::new();
    for (path, checksum) in rows {
        output.push_str(checksum);
        output.push_str("  ");
        output.push_str(path);
        output.push('\n');
    }
    output
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let parent = path
        .parent()
        .ok_or("source lock requires a parent directory")?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("tsv.tmp");
    let mut file = File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};

    fn artifact(bytes: &[u8], signature: &str) -> LockedArtifact {
        let digest = Sha256::digest(bytes);
        LockedArtifact {
            source_id: "fixture".into(),
            artifact_id: "fixture-artifact".into(),
            transport: "direct".into(),
            url: "https://example.invalid/fixture".into(),
            path: "downloads/fixture/value.txt".into(),
            sha256: digest.iter().map(|byte| format!("{byte:02x}")).collect(),
            size_bytes: bytes.len() as u64,
            format: signature.into(),
            signature: signature.into(),
            content_types: "-".into(),
        }
    }

    fn temporary_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "synodal-source-manager-{label}-{}",
            std::process::id()
        ))
    }

    fn serve_once(bytes: &'static [u8], support_range: bool) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("fixture listener");
        let address = listener.local_addr().expect("fixture address");
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("fixture request");
            let mut request = [0_u8; 8192];
            let count = stream.read(&mut request).expect("request bytes");
            let request = String::from_utf8_lossy(&request[..count]);
            let range_start = request.lines().find_map(|line| {
                line.strip_prefix("Range: bytes=")
                    .or_else(|| line.strip_prefix("range: bytes="))
                    .and_then(|value| value.strip_suffix('-'))
                    .and_then(|value| value.parse::<usize>().ok())
            });
            let start = if support_range {
                range_start.unwrap_or_default()
            } else {
                0
            };
            let body = &bytes[start..];
            let status = if start > 0 {
                "HTTP/1.1 206 Partial Content"
            } else {
                "HTTP/1.1 200 OK"
            };
            let content_range = if start > 0 {
                format!(
                    "Content-Range: bytes {start}-{}/{}\r\n",
                    bytes.len() - 1,
                    bytes.len()
                )
            } else {
                String::new()
            };
            let response = format!(
                "{status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n{content_range}Connection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(response.as_bytes()).expect("headers");
            stream.write_all(body).expect("body");
        });
        (format!("http://{address}/fixture"), handle)
    }

    #[test]
    fn lock_round_trip_is_byte_stable() {
        let artifacts = vec![artifact(b"fixture\n", "text")];
        let serialized = serialize_lock(&artifacts);
        assert_eq!(
            parse_lock(serialized.as_bytes()).expect("valid lock"),
            artifacts
        );
        assert_eq!(
            serialize_lock(&parse_lock(serialized.as_bytes()).expect("valid lock")),
            serialized
        );
    }

    #[test]
    fn wrong_checksum_is_not_accepted_as_present() {
        let directory = temporary_directory("checksum");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("artifact.txt");
        fs::write(&path, b"altered\n").expect("fixture");
        let locked = artifact(b"fixture\n", "text");
        assert_eq!(
            inspect_artifact(&path, &locked).expect("inspection"),
            LocalStatus::WrongChecksum
        );
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn html_error_page_cannot_masquerade_as_pdf() {
        let directory = temporary_directory("html-error");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("artifact.pdf");
        fs::write(&path, b"<!doctype html><html>error</html>").expect("fixture");
        for (signature, format) in [
            ("pdf", "pdf"),
            ("zip", "zip"),
            ("gzip", "tar.gz"),
            ("bzip2", "xml.bz2"),
            ("djvu", "djvu"),
            ("xml", "xml"),
        ] {
            assert!(validate_signature(&path, signature, format).is_err());
        }
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn lock_paths_cannot_escape_the_cache() {
        assert!(validate_relative_download_path("downloads/source/file").is_ok());
        assert!(validate_relative_download_path("downloads/../outside").is_err());
        assert!(validate_relative_download_path("/absolute").is_err());
    }

    #[test]
    fn miniature_http_fixture_fetches_and_verifies_atomically() {
        let directory = temporary_directory("http");
        let cache = directory.join("cache");
        let bytes = b"locked fixture\n";
        let (url, server) = serve_once(bytes, false);
        let mut locked = artifact(bytes, "text");
        locked.url = url;
        let destination = artifact_cache_path(&cache, &locked);
        download_locked_artifact(&locked, &cache, &destination).expect("locked download");
        server.join().expect("fixture server");
        assert_eq!(
            inspect_artifact(&destination, &locked).expect("inspection"),
            LocalStatus::Verified
        );
        assert!(!destination.with_extension("txt.partial").exists());
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn interrupted_download_resumes_with_a_range_request() {
        let directory = temporary_directory("resume");
        let cache = directory.join("cache");
        let bytes = b"resumable fixture\n";
        let (url, server) = serve_once(bytes, true);
        let mut locked = artifact(bytes, "text");
        locked.url = url;
        let destination = artifact_cache_path(&cache, &locked);
        let partial = destination.with_extension("txt.partial");
        fs::create_dir_all(partial.parent().expect("partial parent")).expect("parent");
        fs::write(&partial, &bytes[..5]).expect("partial bytes");
        download_locked_artifact(&locked, &cache, &destination).expect("resumed download");
        server.join().expect("fixture server");
        assert_eq!(fs::read(&destination).expect("download"), bytes);
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn mutable_upstream_drift_never_enters_the_cache() {
        let directory = temporary_directory("drift");
        let cache = directory.join("cache");
        let (url, server) = serve_once(b"changed upstream bytes\n", false);
        let mut locked = artifact(b"reviewed locked bytes\n", "text");
        locked.url = url;
        let destination = artifact_cache_path(&cache, &locked);
        assert!(download_locked_artifact(&locked, &cache, &destination).is_err());
        server.join().expect("fixture server");
        assert!(!destination.exists());
        assert!(!destination.with_extension("txt.partial").exists());
        assert!(destination.with_extension("txt.unexpected").exists());
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn fetch_does_not_mutate_source_locks() {
        let directory = temporary_directory("fetch-lock");
        let references = directory.join("references");
        let cache = directory.join("cache");
        fs::create_dir_all(&references).expect("references");
        let bytes = b"locked fixture\n";
        let (url, server) = serve_once(bytes, false);
        let mut locked = artifact(bytes, "text");
        locked.url = url;
        let lock = serialize_lock(std::slice::from_ref(&locked));
        let checksums = serialize_checksums(std::slice::from_ref(&locked));
        fs::write(references.join("SOURCE_LOCK.tsv"), &lock).expect("lock");
        fs::write(references.join("SHA256SUMS"), &checksums).expect("checksums");
        fs::write(
            references.join("SOURCES.toml"),
            "[[source]]\nid = \"fixture\"\nname = \"Fixture\"\n",
        )
        .expect("inventory");
        let mut arguments = vec![
            "fetch".into(),
            "--cache".into(),
            cache.display().to_string(),
        ]
        .into_iter();
        run(&mut arguments, &directory).expect("fixture fetch");
        server.join().expect("fixture server");
        assert_eq!(
            fs::read_to_string(references.join("SOURCE_LOCK.tsv")).expect("lock after"),
            lock
        );
        assert_eq!(
            fs::read_to_string(references.join("SHA256SUMS")).expect("checksums after"),
            checksums
        );
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn offline_verify_does_not_rewrite_the_lock() {
        let directory = temporary_directory("immutable-lock");
        let references = directory.join("references");
        let cache = references.join("downloads");
        fs::create_dir_all(&references).expect("references");
        let bytes = b"fixture\n";
        let locked = artifact(bytes, "text");
        let destination = artifact_cache_path(&cache, &locked);
        fs::create_dir_all(destination.parent().expect("cache parent")).expect("cache");
        fs::write(&destination, bytes).expect("artifact");
        let lock = serialize_lock(std::slice::from_ref(&locked));
        fs::write(references.join("SOURCE_LOCK.tsv"), &lock).expect("lock");
        fs::write(
            references.join("SHA256SUMS"),
            serialize_checksums(std::slice::from_ref(&locked)),
        )
        .expect("checksums");
        fs::write(
            references.join("SOURCES.toml"),
            "[[source]]\nid = \"fixture\"\nname = \"Fixture\"\n",
        )
        .expect("inventory");
        let mut arguments = vec!["verify".into(), "--offline".into()].into_iter();
        run(&mut arguments, &directory).expect("offline verification");
        assert_eq!(
            fs::read_to_string(references.join("SOURCE_LOCK.tsv")).expect("lock after"),
            lock
        );
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn refresh_is_the_only_explicit_checksum_writer() {
        let directory = temporary_directory("refresh-authorization");
        let references = directory.join("references");
        fs::create_dir_all(&references).expect("references");
        let locked = artifact(b"fixture\n", "text");
        let lock = serialize_lock(std::slice::from_ref(&locked));
        let checksums = serialize_checksums(std::slice::from_ref(&locked));
        fs::write(references.join("SOURCE_LOCK.tsv"), &lock).expect("lock");
        fs::write(references.join("SHA256SUMS"), &checksums).expect("checksums");
        fs::write(
            references.join("SOURCES.toml"),
            "[[source]]\nid = \"fixture\"\nname = \"Fixture\"\n",
        )
        .expect("inventory");
        let mut arguments = vec!["refresh".into(), "--source".into(), "fixture".into()].into_iter();
        assert!(run(&mut arguments, &directory).is_err());
        assert_eq!(
            fs::read_to_string(references.join("SOURCE_LOCK.tsv")).expect("lock after"),
            lock
        );
        assert_eq!(
            fs::read_to_string(references.join("SHA256SUMS")).expect("checksums after"),
            checksums
        );
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn info_zip_uses_its_supported_version_probe() {
        assert_eq!(command_version_argument("unzip"), "-v");
        assert_eq!(command_version_argument("tar"), "--version");
    }
}
