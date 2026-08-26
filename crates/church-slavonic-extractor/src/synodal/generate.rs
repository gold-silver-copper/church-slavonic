use std::path::Path;

use super::*;

/// Validates reviewable TSV and atomically writes the generated Rust registry.
pub fn generate_registry(data_directory: &Path, destination: &Path) -> Result<GenerationReport> {
    let lexeme_path = data_directory.join("lexemes.tsv");
    let noun_restriction_path = data_directory.join("noun_restrictions.tsv");
    let principal_path = data_directory.join("principal_parts.tsv");
    let exact_path = data_directory.join("exact_forms.tsv");
    let alignment_path = data_directory.join("alignments.tsv");
    let abbreviation_path = data_directory.join("abbreviations.tsv");
    let abbreviation_family_path = data_directory.join("abbreviation_families.tsv");
    let abbreviation_inventory_path = data_directory.join("abbreviation_inventory.tsv");
    let accent_path = data_directory.join("accents.tsv");
    let accent_paradigm_path = data_directory.join("accent_paradigms.tsv");
    let positional_paradigm_path = data_directory.join("positional_paradigms.tsv");
    let positional_path = data_directory.join("positional_rules.tsv");
    let transformation_path = data_directory.join("transformation_rules.tsv");
    let conflict_path = data_directory.join("conflicts.tsv");
    let irregular_path = data_directory.join("irregular_overrides.tsv");
    let defective_inventory_path = data_directory.join("verb_defectiveness.tsv");
    let irregular_inventory_path = data_directory.join("irregular_verb_inventory.tsv");
    let target_identity_ambiguity_path = data_directory.join("target_identity_ambiguities.tsv");
    let past_classification_review_path = data_directory.join("past_classification_reviews.tsv");
    let v06_exact_review_path = data_directory.join("v06_exact_reviews.tsv");
    let evaluation_path = data_directory.join("evaluation.tsv");

    let mut lexemes = read_table(
        &lexeme_path,
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
        9,
    )?;
    let noun_restrictions = read_table(
        &noun_restriction_path,
        "lexeme_id\tnumber_inventory\tanimacy_inventory\tevidence_id\ttarget_recension",
        5,
    )?;
    let principal_parts = read_table(
        &principal_path,
        "lexeme_id\tsystem\tvalue\tformation\tevidence_id\ttarget_recension",
        6,
    )?;
    let mut exact_forms = read_table(
        &exact_path,
        "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension",
        7,
    )?;
    let alignments = read_table(
        &alignment_path,
        "mapping_id\tsource_lexeme_id\ttarget_lexeme_id\trelation\tstatus\tmorphology\tsemantics\tconfidence_bp\tevidence_ids\ttransformations\treview_note",
        11,
    )?;
    let abbreviations = read_table(
        &abbreviation_path,
        "lexeme_id\tsense_id\tcell\texpanded\tprinted\trule_id\tevidence_id\treversible\trequired_marks\tcontext_restrictions\tambiguity\tsource_recension\ttarget_recension",
        13,
    )?;
    let abbreviation_families = read_table(
        &abbreviation_family_path,
        "lexeme_id\tsense_id\texpanded_prefix\tprinted_prefix\trule_id\tevidence_id\treversible\trequired_marks\tcontext_restrictions\tambiguity\tsource_recension\ttarget_recension",
        12,
    )?;
    let abbreviation_inventory = read_table(
        &abbreviation_inventory_path,
        "source_order\tprinted_head\texpanded_head\tsemantic_scope\tdecision\tlexeme_id\tsense_id\trule_id\tevidence_id\treview_note\ttarget_recension",
        11,
    )?;
    let accents = read_table(
        &accent_path,
        "lexeme_id\tcell\texpanded\taccented\tevidence_id\tsource_id\tsource_recension\ttarget_recension",
        8,
    )?;
    let accent_paradigms = read_table(
        &accent_paradigm_path,
        "lexeme_id\tparadigm_id\tscope\tplacement\tmark\tbreathing\tevidence_id\tsource_id\tcitation\tsource_recension\ttarget_recension",
        11,
    )?;
    let positional_paradigms = read_table(
        &positional_paradigm_path,
        "lexeme_id\tparadigm_id\tscope\toperation\tevidence_id\tsource_id\tcitation\tsource_recension\ttarget_recension",
        9,
    )?;
    let positional_rules = read_table(
        &positional_path,
        "rule_id\tinput\tcontext\toutput\texceptions\tevidence_id\ttarget_recension",
        7,
    )?;
    let transformation_rules = read_table(
        &transformation_path,
        "rule_id\tsource_recension\ttarget_recension\toperation\tstatus\tevidence_id",
        6,
    )?;
    let conflicts = read_table(
        &conflict_path,
        "conflict_id\tsource_lexeme_id\ttarget_lexeme_id\tkind\tstatus\tsupporting_evidence\tcontradicting_evidence\tresolution",
        8,
    )?;
    let irregular_overrides = read_table(
        &irregular_path,
        "lexeme_id\tsystem\tcell_set\tevidence_id\ttarget_recension",
        5,
    )?;
    let defective_inventories = read_table(
        &defective_inventory_path,
        "lexeme_id\tmode\tselector\tkind\tmetadata_field\treason\tevidence_id\ttarget_recension",
        8,
    )?;
    let irregular_inventory = read_table(
        &irregular_inventory_path,
        "source_order\theadword\tsystems\tstrategy\timplementation_status\tevidence_id\tnote\ttarget_recension",
        8,
    )?;
    let reviewed_evidence = read_reviewed_evidence(data_directory)?;
    let past_classification_reviews = read_table(
        &past_classification_review_path,
        PAST_CLASSIFICATION_REVIEW_HEADER,
        10,
    )?;
    let v06_exact_reviews = read_table(&v06_exact_review_path, V06_EXACT_REVIEW_HEADER, 20)?;
    let evaluation = read_table(&evaluation_path, EVALUATION_HEADER, 9)?;
    let lexical_review_path = data_directory.join("lexical_reviews.tsv");
    let lexical_reviews = read_lexical_reviews(data_directory)?;
    let target_identity_ambiguities = read_table(
        &target_identity_ambiguity_path,
        "evidence_id\tcandidate_id\texpanded\tprinted\tleft_lexeme_id\tleft_cells\tright_lexeme_id\tright_cells\tdecision\treview_note",
        10,
    )?;
    validate_target_identity_ambiguities(
        &target_identity_ambiguity_path,
        &target_identity_ambiguities,
        &reviewed_evidence,
    )?;
    validate_lexical_reviews(
        &lexical_review_path,
        &lexical_reviews,
        &target_identity_ambiguities,
    )?;
    let source_recensions = load_source_recensions(data_directory)?;
    let evidence_provenance =
        evidence_provenance_rows(&reviewed_evidence, &lexical_reviews, &source_recensions)?;
    let (review_lexemes, review_exact_forms, _) =
        admitted_lexical_review_rows(&lexical_reviews, &source_recensions)?;
    // A later engine release may add independently reviewed productive
    // metadata for an identity first admitted by a lexical review. Preserve
    // that richer direct row instead of materializing a second exact-only
    // lexeme with the same stable ID.
    extend_missing_lexemes(
        &lexeme_path,
        &mut lexemes,
        review_lexemes,
        &review_exact_forms,
        &exact_forms.rows,
    )?;
    extend_reviewed_exact_forms(&exact_path, &mut exact_forms, review_exact_forms)?;

    validate_lexemes(&lexeme_path, &lexemes)?;
    validate_noun_restrictions(&noun_restriction_path, &noun_restrictions)?;
    validate_noun_restriction_lexemes(&noun_restriction_path, &noun_restrictions, &lexemes)?;
    validate_principal_parts(&principal_path, &principal_parts)?;
    validate_exact_forms(&exact_path, &exact_forms, &lexemes)?;
    validate_noun_restriction_exact_forms(
        &noun_restriction_path,
        &noun_restrictions,
        &exact_path,
        &exact_forms,
    )?;
    validate_past_classification_reviews(
        (
            &past_classification_review_path,
            &past_classification_reviews,
        ),
        (&v06_exact_review_path, &v06_exact_reviews),
        (&exact_path, &exact_forms),
        (&evaluation_path, &evaluation),
    )?;
    validate_absent_target_cells((&exact_path, &exact_forms), (&evaluation_path, &evaluation))?;
    validate_exact_form_attestation_evidence(
        &exact_path,
        &exact_forms,
        &evidence_provenance,
        &reviewed_evidence,
        &lexical_reviews,
        &target_identity_ambiguities,
    )?;
    validate_alignments(&alignment_path, &alignments)?;
    validate_abbreviations(&abbreviation_path, &abbreviations, &lexemes)?;
    validate_abbreviation_families(
        &abbreviation_family_path,
        &abbreviation_families,
        &abbreviations,
        &lexemes,
    )?;
    validate_abbreviation_inventory(
        &abbreviation_inventory_path,
        &abbreviation_inventory,
        &abbreviation_families,
    )?;
    validate_accents(&accent_path, &accents)?;
    validate_accent_paradigms(&accent_paradigm_path, &accent_paradigms)?;
    validate_positional_paradigms(&positional_paradigm_path, &positional_paradigms)?;
    validate_positional_rules(&positional_path, &positional_rules)?;
    validate_transformation_rules(&transformation_path, &transformation_rules)?;
    validate_conflicts(&conflict_path, &conflicts)?;
    validate_conflict_evidence(&conflict_path, &conflicts, &reviewed_evidence)?;
    validate_irregular_overrides(&irregular_path, &irregular_overrides)?;
    merge_irregular_overrides(&irregular_path, &mut exact_forms, &irregular_overrides)?;
    validate_defective_inventories(&defective_inventory_path, &defective_inventories, &lexemes)?;
    validate_irregular_verb_inventory(&irregular_inventory_path, &irregular_inventory)?;
    validate_morphology_evidence(
        data_directory,
        &reviewed_evidence,
        &lexical_reviews,
        [
            (&principal_parts, &[4_usize][..]),
            (&exact_forms, &[4][..]),
            (&alignments, &[8][..]),
            (&abbreviations, &[6][..]),
            (&abbreviation_families, &[5][..]),
            (&abbreviation_inventory, &[8][..]),
            (&accents, &[4][..]),
            (&accent_paradigms, &[6][..]),
            (&positional_paradigms, &[4][..]),
            (&noun_restrictions, &[3][..]),
            (&positional_rules, &[5][..]),
            (&transformation_rules, &[5][..]),
            (&irregular_overrides, &[3][..]),
            (&defective_inventories, &[6][..]),
            (&irregular_inventory, &[5][..]),
        ],
    )?;
    validate_morphology_references(
        &lexeme_path,
        &lexemes,
        [
            (&principal_path, &principal_parts, 0),
            (&exact_path, &exact_forms, 0),
            (&abbreviation_path, &abbreviations, 0),
            (&abbreviation_family_path, &abbreviation_families, 0),
            (&accent_path, &accents, 0),
            (&accent_paradigm_path, &accent_paradigms, 0),
            (&positional_paradigm_path, &positional_paradigms, 0),
            (&noun_restriction_path, &noun_restrictions, 0),
            (&irregular_path, &irregular_overrides, 0),
            (&defective_inventory_path, &defective_inventories, 0),
        ],
    )?;
    validate_alignment_references(
        &alignment_path,
        &alignments,
        &lexemes,
        &transformation_rules,
        &conflict_path,
        &conflicts,
    )?;

    let output = emit_registry(RegistryTables {
        lexemes: lexemes.clone(),
        noun_restrictions: noun_restrictions.clone(),
        principal_parts: principal_parts.clone(),
        exact_forms: exact_forms.clone(),
        alignments: alignments.clone(),
        abbreviations: abbreviations.clone(),
        abbreviation_families: abbreviation_families.clone(),
        accents: accents.clone(),
        accent_paradigms: accent_paradigms.clone(),
        positional_paradigms: positional_paradigms.clone(),
        positional_rules: positional_rules.clone(),
        transformation_rules: transformation_rules.clone(),
        conflicts: conflicts.clone(),
        defective_inventories: defective_inventories.clone(),
        irregular_inventory: irregular_inventory.clone(),
        evidence_provenance,
    });
    let output_sha256 = hex_sha256(output.as_bytes());
    atomic_write(destination, output.as_bytes())?;

    Ok(GenerationReport {
        lexemes: lexemes.rows.len(),
        principal_parts: principal_parts.rows.len(),
        exact_forms: exact_forms.rows.len(),
        accents: accents.rows.len(),
        alignments: alignments.rows.len(),
        abbreviations: abbreviations.rows.len(),
        positional_rules: positional_rules.rows.len(),
        transformation_rules: transformation_rules.rows.len(),
        conflicts: conflicts.rows.len(),
        irregular_overrides: irregular_overrides.rows.len(),
        defective_inventories: defective_inventories.rows.len(),
        irregular_inventory_entries: irregular_inventory.rows.len(),
        output_sha256,
    })
}

/// Validates semantic/reference TSV and writes the dictionary's static registry.
pub fn generate_dictionary_registry(
    data_directory: &Path,
    destination: &Path,
) -> Result<DictionaryGenerationReport> {
    let sense_path = data_directory.join("senses.tsv");
    let example_path = data_directory.join("examples.tsv");
    let semantic_alignment_path = data_directory.join("semantic_alignments.tsv");
    let mut senses = read_table(
        &sense_path,
        "lexeme_id\tsense_id\tgloss\tdomains\tsource_id\tsource_recension\tsemantic_status",
        7,
    )?;
    let examples = read_table(
        &example_path,
        "example_id\tlexeme_id\ttext\ttranslation\tsource_id\tpassage\tsource_recension\ttarget_recension\tpartition",
        9,
    )?;
    let semantic_alignments = read_table(
        &semantic_alignment_path,
        "mapping_id\tsource_sense_id\ttarget_sense_id\tstatus\tevidence_id\treview_note",
        6,
    )?;
    let lexical_review_path = data_directory.join("lexical_reviews.tsv");
    let lexical_reviews = read_lexical_reviews(data_directory)?;
    let reviewed_evidence = read_reviewed_evidence(data_directory)?;
    let target_identity_ambiguity_path = data_directory.join("target_identity_ambiguities.tsv");
    let target_identity_ambiguities = read_table(
        &target_identity_ambiguity_path,
        "evidence_id\tcandidate_id\texpanded\tprinted\tleft_lexeme_id\tleft_cells\tright_lexeme_id\tright_cells\tdecision\treview_note",
        10,
    )?;
    validate_target_identity_ambiguities(
        &target_identity_ambiguity_path,
        &target_identity_ambiguities,
        &reviewed_evidence,
    )?;
    validate_lexical_reviews(
        &lexical_review_path,
        &lexical_reviews,
        &target_identity_ambiguities,
    )?;
    let source_recensions = load_source_recensions(data_directory)?;
    let (review_lexemes, review_exact_forms, review_senses) =
        admitted_lexical_review_rows(&lexical_reviews, &source_recensions)?;
    senses.rows.extend(review_senses);
    validate_senses(&sense_path, &senses, &source_recensions)?;
    validate_examples(&example_path, &examples)?;
    validate_semantic_alignments(&semantic_alignment_path, &semantic_alignments)?;
    validate_semantic_alignment_evidence(
        &semantic_alignment_path,
        &semantic_alignments,
        &reviewed_evidence,
    )?;
    let lexeme_path = data_directory.join("lexemes.tsv");
    let mut lexemes = read_table(
        &lexeme_path,
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
        9,
    )?;
    let exact_forms = read_table(
        &data_directory.join("exact_forms.tsv"),
        "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension",
        7,
    )?;
    extend_missing_lexemes(
        &lexeme_path,
        &mut lexemes,
        review_lexemes,
        &review_exact_forms,
        &exact_forms.rows,
    )?;
    let morphology_alignments = read_table(
        &data_directory.join("alignments.tsv"),
        "mapping_id\tsource_lexeme_id\ttarget_lexeme_id\trelation\tstatus\tmorphology\tsemantics\tconfidence_bp\tevidence_ids\ttransformations\treview_note",
        11,
    )?;
    validate_dictionary_references(
        &sense_path,
        &senses,
        &example_path,
        &examples,
        &semantic_alignment_path,
        &semantic_alignments,
        &lexemes,
        &morphology_alignments,
    )?;

    let output = emit_dictionary_registry(
        senses.clone(),
        examples.clone(),
        semantic_alignments.clone(),
    );
    let output_sha256 = hex_sha256(output.as_bytes());
    atomic_write(destination, output.as_bytes())?;
    Ok(DictionaryGenerationReport {
        senses: senses.rows.len(),
        examples: examples.rows.len(),
        semantic_alignments: semantic_alignments.rows.len(),
        output_sha256,
    })
}

/// True when an irregular-override `system` label covers the exact-form
/// `cell` key. This is the single place the label-to-cell mapping lives;
/// the runtime consumes the stamped column instead of re-deriving it.
fn irregular_system_covers(system: &str, cell: &str) -> bool {
    match system {
        "present" | "future" | "aorist" | "imperfect" | "imperative" => {
            cell.starts_with(system) && cell.as_bytes().get(system.len()) == Some(&b':')
        }
        "noun-singular-dative-and-plural" => {
            cell.starts_with("noun:dative:singular:") || cell.contains(":plural:")
        }
        _ => false,
    }
}

/// Folds `irregular_overrides.tsv` into the exact-form table the registry
/// emits: every covered exact row gains two provenance columns
/// (`irregular_system`, `irregular_evidence_id`), so the runtime consults one
/// merged irregular table while the trace still names the reviewed override
/// evidence. Uncovered rows carry empty markers. An override that stamps no
/// row is an error: it would silently vanish from the merged table.
fn merge_irregular_overrides(
    irregular_path: &Path,
    exact_forms: &mut Table,
    irregular_overrides: &Table,
) -> Result<()> {
    for row in &mut exact_forms.rows {
        row.push(String::new());
        row.push(String::new());
    }
    for (offset, override_row) in irregular_overrides.rows.iter().enumerate() {
        let mut stamped = 0_usize;
        for exact_row in &mut exact_forms.rows {
            if exact_row[0] != override_row[0]
                || !irregular_system_covers(&override_row[1], &exact_row[1])
            {
                continue;
            }
            if !exact_row[7].is_empty() && exact_row[7] != override_row[1] {
                return Err(ExtractionError::InvalidRow {
                    file: irregular_path.to_owned(),
                    line: offset + 2,
                    reason: format!(
                        "irregular override systems {} and {} both cover exact cell {}",
                        exact_row[7], override_row[1], exact_row[1]
                    ),
                });
            }
            exact_row[7] = override_row[1].clone();
            exact_row[8] = override_row[3].clone();
            stamped += 1;
        }
        if stamped == 0 {
            return Err(ExtractionError::InvalidRow {
                file: irregular_path.to_owned(),
                line: offset + 2,
                reason: format!(
                    "irregular override for {} system {} covers no exact-form row",
                    override_row[0], override_row[1]
                ),
            });
        }
    }
    Ok(())
}
