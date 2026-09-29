//! Compiled cache transport, accepted only against a caller-trusted byte digest.
//! The digest binds compiler output, not historical truth. Loading validates
//! structure/references and compatibility; it does not regenerate the paradigm
//! inventory to prove completeness independently of the trusted compiler.
use super::*;
use std::io::Write;

pub const MAX_COMPILED_INDEX_BYTES: usize = 64 * 1024 * 1024;
const MAX_RECORD_BYTES: usize = 1024 * 1024;
const MAX_RECORD_VALUES: usize = 4096;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    version: u32,
    model_sha256: String,
    engine_sha256: String,
    runtime_sha256: String,
    generation_policy_sha256: String,
    orthography: OrthographyId,
    policy: usize,
    forms: usize,
    restrictions: usize,
    missing: usize,
    construction_checks: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingRestriction {
    lexeme: LexemeId,
    restriction: Restriction,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Missing {
    lexeme: LexemeId,
    #[serde(with = "super::super::cell_codec")]
    cell: Cell,
    stems: Vec<String>,
}
#[derive(Deserialize)]
enum Record {
    Form(Box<IndexedForm>),
    Restriction(PendingRestriction),
    Missing(Missing),
}
#[derive(Serialize)]
enum RecordRef<'a> {
    Form(&'a IndexedForm),
    Restriction {
        lexeme: &'a LexemeId,
        restriction: &'a Restriction,
    },
    Missing {
        lexeme: &'a LexemeId,
        #[serde(with = "super::super::cell_codec")]
        cell: Cell,
        stems: &'a [String],
    },
}
fn invalid(message: &'static str) -> ModelError {
    ModelError::ArtifactMismatch(message)
}
fn digest_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// Enforce nesting and value-count bounds before Serde can allocate collections.
// This scanner establishes bounds only; Serde still validates JSON/escapes/types.
fn check_record(bytes: &[u8]) -> Result<(), ModelError> {
    if bytes.is_empty() || bytes.len() > MAX_RECORD_BYTES {
        return Err(ModelError::ResourceLimit("compiled index record bytes"));
    }
    let (mut quoted, mut escaped, mut depth, mut values) = (false, false, 0usize, 0usize);
    for &b in bytes {
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
        } else {
            match b {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    depth += 1;
                    values += 1;
                }
                b'}' | b']' => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or_else(|| invalid("JSON nesting"))?;
                }
                b',' => values += 1,
                _ => {}
            }
        }
        if depth > 64 || values > MAX_RECORD_VALUES {
            return Err(ModelError::ResourceLimit("compiled index JSON structure"));
        }
    }
    if quoted || depth != 0 {
        return Err(invalid("incomplete JSON record"));
    }
    Ok(())
}
fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, ModelError> {
    check_record(bytes)?;
    serde_json::from_slice(bytes).map_err(|e| ModelError::Encoding(e.to_string()))
}
struct Output {
    bytes: Vec<u8>,
    record_start: usize,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let size = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|&n| n <= MAX_COMPILED_INDEX_BYTES && n - self.record_start <= MAX_RECORD_BYTES)
            .ok_or_else(|| std::io::Error::other("compiled index output limit"))?;
        self.bytes.reserve(size - self.bytes.len());
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Output {
    fn record(&mut self, value: &impl Serialize) -> Result<(), ModelError> {
        self.record_start = self.bytes.len();
        serde_json::to_writer(&mut *self, value)
            .map_err(|e| ModelError::Encoding(e.to_string()))?;
        check_record(&self.bytes[self.record_start..])?;
        self.write_all(b"\n")
            .map_err(|e| ModelError::Encoding(e.to_string()))
    }
}
impl Model {
    /// Compile an index under a caller-supplied runtime/build identity. A caller
    /// must retain the resulting byte digest through a trusted channel. The
    /// artifact's own header cannot authorize loading or establish completeness.
    pub fn compile_analysis_index(
        &self,
        orthography: &OrthographyId,
        policy: MatchPolicy,
        runtime_sha256: &str,
    ) -> Result<Vec<u8>, ModelError> {
        if !digest_valid(runtime_sha256) {
            return Err(invalid("runtime identity format"));
        }
        let cache_key = (orthography.clone(), policy.slot());
        let mut cache = self.analysis_cache.write().map_err(|_| poisoned())?;
        if !cache.entries.contains_key(&cache_key) {
            let built = Prepared::build(self, orthography, policy, &cache.stats)?;
            cache.stats.indexes += 1;
            cache.stats.forms += built.stats.forms;
            cache.stats.retained_bytes += built.stats.retained_bytes;
            cache.stats.construction_checks += built.stats.construction_checks;
            cache.entries.insert(cache_key.clone(), built);
        }
        let index = &cache.entries[&cache_key];
        let mut output = Output {
            bytes: Vec::new(),
            record_start: 0,
        };
        output.record(&Header {
            version: 1,
            model_sha256: self.data_sha256().into(),
            engine_sha256: self.engine_sha256().into(),
            runtime_sha256: runtime_sha256.into(),
            generation_policy_sha256: self.generation_policy_sha256().into(),
            orthography: orthography.clone(),
            policy: policy.slot(),
            forms: index.stats.forms,
            restrictions: index.restrictions.len(),
            missing: index.missing.len(),
            construction_checks: index.stats.construction_checks,
        })?;
        // The retrieval key is reconstructed on load, not trusted from disk.
        for forms in index.forms.values() {
            for form in forms {
                output.record(&RecordRef::Form(form))?;
            }
        }
        for (lexeme, restriction) in &index.restrictions {
            output.record(&RecordRef::Restriction {
                lexeme,
                restriction,
            })?;
        }
        for (lexeme, cell, stems) in &index.missing {
            output.record(&RecordRef::Missing {
                lexeme,
                cell: *cell,
                stems,
            })?;
        }
        Ok(output.bytes)
    }

    /// Install trusted compiler output before sharing the model. No generation
    /// occurs here. The expected digest and runtime identity come from the
    /// caller, never from an adjacent untrusted manifest or this payload itself.
    pub fn install_compiled_analysis_index(
        &mut self,
        bytes: &[u8],
        expected_sha256: &str,
        runtime_sha256: &str,
        orthography: &OrthographyId,
        policy: MatchPolicy,
    ) -> Result<AnalysisIndexStats, ModelError> {
        if bytes.len() > MAX_COMPILED_INDEX_BYTES {
            return Err(ModelError::ResourceLimit("compiled index bytes"));
        }
        if !digest_valid(expected_sha256)
            || format!("{:x}", Sha256::digest(bytes)) != expected_sha256
        {
            return Err(invalid("compiled index digest"));
        }
        if !digest_valid(runtime_sha256) {
            return Err(invalid("runtime identity format"));
        }
        if bytes.last() != Some(&b'\n') {
            return Err(invalid("compiled index final newline"));
        }
        let mut lines = bytes[..bytes.len() - 1].split(|&b| b == b'\n');
        let header: Header = parse(lines.next().ok_or_else(|| invalid("missing header"))?)?;
        if header.version != 1 {
            return Err(invalid("compiled index version"));
        }
        if header.model_sha256 != self.data_sha256() {
            return Err(invalid("compiled index model"));
        }
        if header.engine_sha256 != self.engine_sha256() {
            return Err(invalid("compiled index engine"));
        }
        if header.runtime_sha256 != runtime_sha256 {
            return Err(invalid("compiled index runtime"));
        }
        if header.generation_policy_sha256 != self.generation_policy_sha256() {
            return Err(invalid("compiled index generation policy"));
        }
        if &header.orthography != orthography || header.policy != policy.slot() {
            return Err(invalid("compiled index profile/policy"));
        }
        let spelling = self
            .orthography(orthography)
            .ok_or_else(|| ModelError::MissingReference(orthography.0.clone()))?;
        let cache = self.analysis_cache.read().map_err(|_| poisoned())?;
        let cache_key = (orthography.clone(), policy.slot());
        if cache.entries.contains_key(&cache_key) {
            return Err(invalid("compiled index already installed"));
        }
        let bytes_limit = self
            .limits
            .max_index_bytes
            .saturating_sub(cache.stats.retained_bytes);
        let forms_limit = self
            .limits
            .max_index_forms
            .saturating_sub(cache.stats.forms);
        let work_limit = self
            .limits
            .max_index_checks
            .saturating_sub(cache.stats.construction_checks);
        if header.forms > forms_limit
            || header.construction_checks > work_limit
            || header.restrictions > work_limit
            || header.missing > work_limit
        {
            return Err(ModelError::ResourceLimit("compiled index counts"));
        }
        drop(cache);
        let expected_records = header
            .forms
            .checked_add(header.restrictions)
            .and_then(|n| n.checked_add(header.missing))
            .ok_or_else(|| invalid("compiled index record count"))?;
        let mut out = Prepared::default();
        charge(
            &mut out.stats.retained_bytes,
            std::mem::size_of::<Prepared>() + orthography.0.len(),
            bytes_limit,
            "analysis index bytes",
        )?;
        let mut seen_forms = BTreeSet::new();
        let mut seen_restrictions = BTreeSet::new();
        let mut seen_missing = BTreeSet::new();
        let mut records = 0;
        let mut work = 0;
        for line in lines {
            charge(&mut records, 1, expected_records, "compiled index records")?;
            charge(
                &mut work,
                1,
                self.limits.max_index_checks,
                "compiled index validation",
            )?;
            match parse::<Record>(line)? {
                Record::Form(form) => {
                    let mut form = *form;
                    if matches!(
                        form.derivation.review,
                        GenerationReview::CallerAccepted { .. }
                    ) {
                        // Operational trust in a compiled file is not permission
                        // to attach a review decision to a different derivation.
                        let claimed = std::mem::take(&mut form.derivation.claim_sha256);
                        let reviewed = std::mem::take(&mut form.derivation.review);
                        self.qualify_generation(
                            &form.lexeme,
                            form.cell,
                            &mut form.derivation,
                            self.limits.max_result_bytes,
                        )?;
                        if form.derivation.claim_sha256 != claimed
                            || form.derivation.review != reviewed
                        {
                            return Err(invalid("compiled accepted claim content"));
                        }
                    }
                    let entry = self
                        .lexeme(&form.lexeme)
                        .ok_or_else(|| ModelError::MissingReference(form.lexeme.0.clone()))?;
                    let paradigm = &self.paradigms[&entry.paradigm];
                    let d = &form.derivation;
                    let review_matches =
                        match (&d.review, self.accepted_generation.get(&d.claim_sha256)) {
                            (GenerationReview::Unreviewed, None) => true,
                            (
                                GenerationReview::CallerAccepted {
                                    decision_id,
                                    method,
                                },
                                Some(accepted),
                            ) => {
                                decision_id == &accepted.decision_id
                                    && method == &accepted.method
                                    && accepted.lexeme == form.lexeme
                                    && accepted.cell == form.cell
                                    && accepted.orthography == *orthography
                            }
                            _ => false,
                        };
                    if !review_matches {
                        return Err(invalid("compiled generation review"));
                    }

                    charge(
                        &mut work,
                        paradigm.rules.len() + d.evidence.len() + d.source_annotations.len(),
                        self.limits.max_index_checks,
                        "compiled index validation",
                    )?;
                    if d.orthography != *orthography
                        || d.paradigm != entry.paradigm
                        || d.grammar != paradigm.grammar
                        || !spelling.compatible_grammars.contains(&d.grammar)
                        || !paradigm
                            .rules
                            .iter()
                            .any(|r| r.id == d.rule && r.cell == form.cell)
                        || d.surface.len() > self.limits.max_form_bytes
                        || !digest_valid(&d.claim_sha256)
                        || d.evidence.iter().any(|id| self.evidence(id).is_none())
                        || d.source_annotations
                            .iter()
                            .any(|id| self.observations.annotation(id).is_none())
                        || !seen_forms.insert((
                            form.lexeme.clone(),
                            form.cell,
                            d.claim_sha256.clone(),
                        ))
                    {
                        return Err(invalid("compiled form references/identity"));
                    }
                    charge(&mut out.stats.forms, 1, forms_limit, "analysis index forms")?;
                    let retrieval = key(&d.surface, policy);
                    let size = std::mem::size_of::<IndexedForm>()
                        + form.lexeme.0.len()
                        + d.retained_bytes()
                        + std::mem::size_of::<(String, Vec<IndexedForm>)>()
                        + retrieval.len();
                    charge(
                        &mut out.stats.retained_bytes,
                        size,
                        bytes_limit,
                        "analysis index bytes",
                    )?;
                    out.forms.entry(retrieval).or_default().push(form);
                }
                Record::Restriction(row) => {
                    let entry = self
                        .lexeme(&row.lexeme)
                        .ok_or_else(|| ModelError::MissingReference(row.lexeme.0.clone()))?;
                    if !entry.unavailable.contains(&row.restriction)
                        || !seen_restrictions.insert((row.lexeme.clone(), row.restriction.cell))
                    {
                        return Err(invalid("compiled restriction references"));
                    }
                    let size = std::mem::size_of::<(LexemeId, Restriction)>()
                        + row.lexeme.0.len()
                        + row.restriction.reason.len()
                        + row
                            .restriction
                            .evidence
                            .iter()
                            .map(|e| std::mem::size_of::<EvidenceId>() + e.0.len())
                            .sum::<usize>();
                    charge(
                        &mut out.stats.retained_bytes,
                        size,
                        bytes_limit,
                        "analysis index bytes",
                    )?;
                    out.restrictions.push((row.lexeme, row.restriction));
                }
                Record::Missing(row) => {
                    let entry = self
                        .lexeme(&row.lexeme)
                        .ok_or_else(|| ModelError::MissingReference(row.lexeme.0.clone()))?;
                    let paradigm = &self.paradigms[&entry.paradigm];
                    charge(
                        &mut work,
                        paradigm.rules.len(),
                        self.limits.max_index_checks,
                        "compiled index validation",
                    )?;
                    if !spelling.compatible_grammars.contains(&paradigm.grammar)
                        || !paradigm.rules.iter().any(|r| r.cell == row.cell)
                        || row.stems.is_empty()
                        || row.stems.iter().any(|s| {
                            !paradigm
                                .rules
                                .iter()
                                .any(|r| r.cell == row.cell && &r.stem == s)
                                || entry.stems.iter().any(|stem| &stem.name == s)
                        })
                        || !seen_missing.insert((row.lexeme.clone(), row.cell))
                    {
                        return Err(invalid("compiled missing-stem references"));
                    }
                    out.add_missing(entry, row.cell, row.stems, bytes_limit)?;
                }
            }
        }
        if records != expected_records
            || out.stats.forms != header.forms
            || out.restrictions.len() != header.restrictions
            || out.missing.len() != header.missing
        {
            return Err(invalid("compiled index count mismatch"));
        }
        // Compiler emits each bucket in stable generation order; preserving that
        // order also preserves alternatives of one rule/cell. Digest binds it.
        out.stats.indexes = 1;
        out.stats.restored_indexes = 1;
        out.stats.construction_checks = header.construction_checks;
        let cache = self.analysis_cache.get_mut().map_err(|_| poisoned())?;
        cache.stats.indexes += 1;
        cache.stats.restored_indexes += 1;
        cache.stats.forms += out.stats.forms;
        cache.stats.retained_bytes += out.stats.retained_bytes;
        cache.stats.construction_checks += out.stats.construction_checks;
        cache.entries.insert(cache_key, out);
        Ok(cache.stats.clone())
    }
}
