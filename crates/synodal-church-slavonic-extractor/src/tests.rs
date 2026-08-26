use std::{collections::BTreeMap, path::Path};

use super::*;

#[test]
fn reusable_pronoun_accent_scopes_are_reviewable() {
    let path = Path::new("accent_paradigms.tsv");
    // The runtime registry compiles both pronoun scope shapes, so the
    // reviewed data layer must accept exactly the same two grammars.
    assert!(
        validate_accent_scope_code(path, 2, "pronoun:singular,plural:nominative,genitive").is_ok()
    );
    assert!(
        validate_accent_scope_code(
            path,
            2,
            "pronoun-agreeing:singular:nominative:masculine,neuter:animate,inanimate"
        )
        .is_ok()
    );
}

#[test]
fn malformed_pronoun_accent_scopes_are_rejected() {
    let path = Path::new("accent_paradigms.tsv");
    assert!(validate_accent_scope_code(path, 2, "pronoun:quadral:nominative").is_err());
    assert!(validate_accent_scope_code(path, 2, "pronoun:singular:ergative").is_err());
    assert!(
        validate_accent_scope_code(
            path,
            2,
            "pronoun-agreeing:singular:nominative:common:animate"
        )
        .is_err()
    );
    assert!(
        validate_accent_scope_code(
            path,
            2,
            "pronoun-agreeing:singular:nominative:masculine:sentient"
        )
        .is_err()
    );
    // The bare two-part form has no registry counterpart and stays invalid.
    assert!(validate_accent_scope_code(path, 2, "pronoun:singular").is_err());
}

#[test]
fn independent_future_principal_parts_are_atomic() {
    let path = Path::new("principal_parts.tsv");
    let row = |system: &str, value: &str| {
        vec![
            "synodal:verb:test".into(),
            system.into(),
            value.into(),
            String::new(),
            "test-evidence".into(),
            TARGET.into(),
        ]
    };
    let partial = Table {
        rows: vec![row("future-stem", "возм")],
    };
    assert!(validate_principal_parts(path, &partial).is_err());

    let complete = Table {
        rows: vec![
            row("future-stem", "возм"),
            row("future-first-singular", "возмꙋ"),
            row("future-third-plural", "возмꙋтъ"),
        ],
    };
    validate_principal_parts(path, &complete).expect("complete future triple");
}

#[test]
fn rust_string_emission_is_lossless() {
    let mut output = String::new();
    push_rust_string(&mut output, "слово\\\"\t");
    assert_eq!(output, "\"слово\\\\\\\"\\t\"");
}

#[test]
fn rejects_forbidden_authority_names() {
    let error = reject_forbidden_authority(Path::new("source.tsv"), 2, "Slovowiki")
        .expect_err("forbidden authority");
    assert!(error.to_string().contains("forbidden"));
}

#[test]
fn source_approval_fails_closed_for_neutral_ids() {
    assert!(!source_recension_is_approved(
        "unreviewed-neutral-source",
        "mixed"
    ));
    assert!(source_recension_is_approved(
        "ponomar-elizabeth-bible-2026-08-09",
        TARGET
    ));
}

#[test]
fn lexical_attestations_require_a_direct_target_corpus() {
    let source_recensions = BTreeMap::from([
        ("neutral-comparative-source".into(), "mixed".into()),
        ("ponomar-elizabeth-bible-2026-08-09".into(), TARGET.into()),
    ]);
    assert!(
        require_direct_target_source("neutral-comparative-source", &source_recensions).is_err()
    );
    assert_eq!(
        require_direct_target_source("ponomar-elizabeth-bible-2026-08-09", &source_recensions)
            .expect("approved target source"),
        TARGET
    );
}

#[test]
fn lexical_reviews_may_use_an_independently_established_typed_cell() {
    let mut row = vec![
        "review:typed-noun".into(),
        "synodal:noun:typed".into(),
        "sense:typed-noun".into(),
        "жрецъ".into(),
        "noun".into(),
        "noun:nominative:singular:animate".into(),
        "жрецъ".into(),
        "жре́цъ".into(),
        "priest".into(),
        "person".into(),
        "semantic-source".into(),
        "synodal:candidate:typed-semantic".into(),
        "ponomar-elizabeth-bible-2026-08-09".into(),
        "synodal:candidate:typed-attestation".into(),
        "Amos.7.10".into(),
        "reviewed".into(),
        TARGET.into(),
        "independently typed nominative witness".into(),
    ];
    validate_lexical_reviews(
        Path::new("lexical_reviews.tsv"),
        &Table {
            rows: vec![row.clone()],
        },
        &Table { rows: vec![] },
    )
    .expect("matching typed noun cell");

    row[5] = "infinitive".into();
    assert!(
        validate_lexical_reviews(
            Path::new("lexical_reviews.tsv"),
            &Table {
                rows: vec![row.clone()],
            },
            &Table { rows: vec![] },
        )
        .is_err()
    );

    row[1] = "synodal:adverb:typed".into();
    row[4] = "adverb".into();
    row[5] = "lexical-form".into();
    assert!(
        validate_lexical_reviews(
            Path::new("lexical_reviews.tsv"),
            &Table {
                rows: vec![row.clone()],
            },
            &Table { rows: vec![] },
        )
        .is_err()
    );
    row[5] = "indeclinable".into();
    validate_lexical_reviews(
        Path::new("lexical_reviews.tsv"),
        &Table { rows: vec![row] },
        &Table { rows: vec![] },
    )
    .expect("closed lexical reviews remain indeclinable");

    let mut participle_row = vec![
        "review:typed-participle".into(),
        "synodal:participle:typed".into(),
        "sense:typed-participle".into(),
        "бывъ".into(),
        "participle".into(),
        "participle:past:active:nominative:singular:masculine:inanimate:short:positive".into(),
        "бывъ".into(),
        "бы́въ".into(),
        "having been".into(),
        "grammar".into(),
        "semantic-source".into(),
        "synodal:candidate:typed-participle-semantic".into(),
        "ponomar-elizabeth-bible-2026-08-09".into(),
        "synodal:candidate:typed-participle-attestation".into(),
        "Passage.1".into(),
        "reviewed".into(),
        TARGET.into(),
        "independently typed participle witness".into(),
    ];
    validate_lexical_reviews(
        Path::new("lexical_reviews.tsv"),
        &Table {
            rows: vec![participle_row.clone()],
        },
        &Table { rows: vec![] },
    )
    .expect("matching typed participle cell");
    participle_row[5] = "noun:nominative:singular:animate".into();
    assert!(
        validate_lexical_reviews(
            Path::new("lexical_reviews.tsv"),
            &Table {
                rows: vec![participle_row],
            },
            &Table { rows: vec![] },
        )
        .is_err()
    );
}

#[test]
fn rejects_unreviewed_or_unproven_recension_mappings() {
    let path = Path::new("alignments.tsv");
    let unknown_status = Table {
        rows: vec![vec![
            "map:test".into(),
            "ocs:test".into(),
            "synodal:test".into(),
            "inherited-from".into(),
            "guessed".into(),
            "compatible".into(),
            "established".into(),
            "9000".into(),
            "evidence".into(),
            "identity-test".into(),
            "fixture".into(),
        ]],
    };
    assert!(validate_alignments(path, &unknown_status).is_err());

    let no_evidence = Table {
        rows: vec![vec![
            "map:test".into(),
            "ocs:test".into(),
            "synodal:test".into(),
            "inherited-from".into(),
            "reviewed".into(),
            "compatible".into(),
            "established".into(),
            "9000".into(),
            String::new(),
            "identity-test".into(),
            "fixture".into(),
        ]],
    };
    assert!(validate_alignments(path, &no_evidence).is_err());
}

#[test]
fn rejects_other_recensions_and_unreviewed_runtime_evidence() {
    assert!(validate_target(Path::new("exact_forms.tsv"), 2, "old-church-slavonic").is_err());
    let reviewed = Table {
        rows: vec![vec![
            "known-evidence".into(),
            "synodal:candidate:known".into(),
            "source".into(),
            "citation".into(),
            "reviewed".into(),
            TARGET.into(),
            "review".into(),
        ]],
    };
    let runtime = Table {
        rows: vec![vec!["lexeme".into(), "missing-evidence".into()]],
    };
    let lexical_reviews = Table { rows: Vec::new() };
    assert!(
        validate_morphology_evidence(
            Path::new("data/synodal"),
            &reviewed,
            &lexical_reviews,
            [(&runtime, &[1_usize][..])],
        )
        .is_err()
    );

    let rejected = Table {
        rows: vec![vec![
            "rejected-evidence".into(),
            "synodal:candidate:rejected".into(),
            "source".into(),
            "citation".into(),
            "rejected".into(),
            TARGET.into(),
            "review".into(),
        ]],
    };
    let rejected_runtime = Table {
        rows: vec![vec!["lexeme".into(), "rejected-evidence".into()]],
    };
    assert!(
        validate_morphology_evidence(
            Path::new("data/synodal"),
            &rejected,
            &lexical_reviews,
            [(&rejected_runtime, &[1_usize][..])],
        )
        .is_err()
    );
}

#[test]
fn exact_candidate_matching_accepts_canonical_unicode_equivalence() {
    let candidate = CandidateLink {
        source_id: "ponomar-elizabeth-bible-2026-08-09".into(),
        target_recension: Some(TARGET.into()),
        partition: Some("source".into()),
        passage: Some("Acts.1.10".into()),
        raw_spelling: "и҆ сѐ, мꙋ̑жа два̀".into(),
        normalized_spelling: String::new(),
    };
    assert!(candidate.contains_exact("сѐ"));
    assert!(
        !CandidateLink {
            raw_spelling: "вѣ́рꙋеши".into(),
            ..candidate
        }
        .contains_exact("вѣ́рꙋ")
    );
}

#[test]
fn productive_lexical_upgrades_must_preserve_reviewed_identity() {
    let productive = vec![
        "synodal:noun:test".into(),
        "имѧ".into(),
        "noun".into(),
        "fourth-neuter-en".into(),
        "имен".into(),
        "neuter".into(),
        String::new(),
        "grammar".into(),
        TARGET.into(),
    ];
    let reviewed = vec![
        "synodal:noun:test".into(),
        "небо".into(),
        "noun".into(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        "attestation".into(),
        TARGET.into(),
    ];
    let mut lexemes = Table {
        rows: vec![productive.clone()],
    };
    let error = extend_missing_lexemes(
        Path::new("lexemes.tsv"),
        &mut lexemes,
        vec![reviewed],
        &[],
        &[],
    )
    .expect_err("identity mismatch must fail closed");
    assert!(
        error
            .to_string()
            .contains("must preserve the reviewed source or exact target citation")
    );

    let mut compatible = productive;
    compatible[3] = String::new();
    compatible[4] = String::new();
    compatible[5] = String::new();
    compatible[7] = "attestation".into();
    extend_missing_lexemes(
        Path::new("lexemes.tsv"),
        &mut lexemes,
        vec![compatible],
        &[],
        &[],
    )
    .expect("matching reviewed identity");
    assert_eq!(lexemes.rows.len(), 1);
    assert_eq!(lexemes.rows[0][3], "fourth-neuter-en");

    let target = vec![
        "synodal:noun:stone".into(),
        "камень".into(),
        "noun".into(),
        "fourth-masculine-en-kamen".into(),
        "камен".into(),
        "masculine".into(),
        String::new(),
        "grammar".into(),
        TARGET.into(),
    ];
    let source = vec![
        "synodal:noun:stone".into(),
        "камꙑ".into(),
        "noun".into(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        "attestation".into(),
        TARGET.into(),
    ];
    let exact = vec![
        "synodal:noun:stone".into(),
        "lexical-form".into(),
        "камень".into(),
        "Ка́мень".into(),
        "review".into(),
        "synodal-attestation".into(),
        TARGET.into(),
    ];
    let alternate_exact = vec![
        "synodal:noun:stone".into(),
        "lexical-form".into(),
        "камы".into(),
        "Ка́мы".into(),
        "review".into(),
        "synodal-attestation".into(),
        TARGET.into(),
    ];
    let mut target_lexemes = Table { rows: vec![target] };
    extend_missing_lexemes(
        Path::new("lexemes.tsv"),
        &mut target_lexemes,
        vec![source],
        &[],
        &[alternate_exact, exact],
    )
    .expect("committed exact target citation preserves the stable identity");
    assert_eq!(target_lexemes.rows.len(), 1);
}

#[test]
fn lexeme_closed_codes_fail_before_registry_generation() {
    let valid = vec![
        "synodal:noun:test".into(),
        "камень".into(),
        "noun".into(),
        "fourth-masculine-en-kamen".into(),
        "камен".into(),
        "masculine".into(),
        String::new(),
        "grammar".into(),
        TARGET.into(),
    ];
    for (column, invalid_value) in [
        (2, "unknown-pos"),
        (3, "unknown-class"),
        (5, "unknown-gender"),
        (6, "unknown-aspect"),
    ] {
        let mut row = valid.clone();
        row[column] = invalid_value.into();
        assert!(
            validate_lexemes(Path::new("lexemes.tsv"), &Table { rows: vec![row] }).is_err(),
            "column {column} must be closed"
        );
    }
}

#[test]
fn grammar_cell_rows_fail_with_source_context_before_emission() {
    let path = Path::new("exact_forms.tsv");
    let error = validate_grammar_cell(path, 17, "noun:ablative:singular:inanimate")
        .expect_err("unknown case must fail before registry emission");
    assert!(matches!(
        error,
        ExtractionError::InvalidRow {
            file,
            line: 17,
            ..
        } if file == path
    ));
    validate_grammar_cell(
        Path::new("abbreviations.tsv"),
        2,
        "pronoun:nominative:singular:any:any",
    )
    .expect("legacy wildcard cells remain accepted");
}

#[test]
fn exact_and_abbreviation_cells_must_match_lexeme_part_of_speech() {
    let lexemes = Table {
        rows: vec![vec![
            "synodal:noun:test".into(),
            "градъ".into(),
            "noun".into(),
            "first-hard-m".into(),
            "град".into(),
            "masculine".into(),
            String::new(),
            "test-source".into(),
            TARGET.into(),
        ]],
    };
    let exact = Table {
        rows: vec![vec![
            "synodal:noun:test".into(),
            "indeclinable".into(),
            "градъ".into(),
            "гра́дъ".into(),
            "test-evidence".into(),
            "normative-table".into(),
            TARGET.into(),
        ]],
    };
    assert!(validate_exact_forms(Path::new("exact_forms.tsv"), &exact, &lexemes).is_err());

    let abbreviations = Table {
        rows: vec![vec![
            "synodal:noun:test".into(),
            "sense:test".into(),
            "indeclinable".into(),
            "градъ".into(),
            "гра́дъ".into(),
            "test-rule".into(),
            "test-evidence".into(),
            "true".into(),
            "titlo".into(),
            "unrestricted".into(),
            "unambiguous".into(),
            TARGET.into(),
            TARGET.into(),
        ]],
    };
    assert!(
        validate_abbreviations(Path::new("abbreviations.tsv"), &abbreviations, &lexemes,).is_err()
    );
}

#[test]
fn abbreviation_families_must_reproduce_every_reviewed_exact_shape() {
    let lexemes = Table {
        rows: vec![vec![
            "synodal:noun:test".into(),
            "градъ".into(),
            "noun".into(),
            "first-hard-m".into(),
            "град".into(),
            "masculine".into(),
            String::new(),
            "test-source".into(),
            TARGET.into(),
        ]],
    };
    let abbreviations = Table {
        rows: vec![vec![
            "synodal:noun:test".into(),
            "sense:test".into(),
            "noun:nominative:singular:inanimate".into(),
            "градъ".into(),
            "гр҃дъ".into(),
            "test-exact-rule".into(),
            "test-evidence".into(),
            "false".into(),
            "titlo".into(),
            "test identity".into(),
            "non-reversible".into(),
            TARGET.into(),
            TARGET.into(),
        ]],
    };
    let mut family_row = vec![
        "synodal:noun:test".into(),
        "sense:test".into(),
        "гра".into(),
        "гр҃".into(),
        "test-family-rule".into(),
        "test-evidence".into(),
        "false".into(),
        "titlo".into(),
        "test identity".into(),
        "non-reversible".into(),
        TARGET.into(),
        TARGET.into(),
    ];
    validate_abbreviation_families(
        Path::new("abbreviation_families.tsv"),
        &Table {
            rows: vec![family_row.clone()],
        },
        &abbreviations,
        &lexemes,
    )
    .expect("matching family skeleton");

    let mut unused = family_row.clone();
    unused[2] = "гро".into();
    assert!(
        validate_abbreviation_families(
            Path::new("abbreviation_families.tsv"),
            &Table {
                rows: vec![family_row.clone(), unused],
            },
            &abbreviations,
            &lexemes,
        )
        .is_err()
    );

    family_row[3] = "гд҃".into();
    assert!(
        validate_abbreviation_families(
            Path::new("abbreviation_families.tsv"),
            &Table {
                rows: vec![family_row],
            },
            &abbreviations,
            &lexemes,
        )
        .is_err()
    );
}

#[test]
fn exact_runtime_tuples_must_be_unique() {
    let lexemes = Table {
        rows: vec![vec![
            "synodal:noun:test".into(),
            "градъ".into(),
            "noun".into(),
            "first-hard-m".into(),
            "град".into(),
            "masculine".into(),
            String::new(),
            "test-source".into(),
            TARGET.into(),
        ]],
    };
    let row = vec![
        "synodal:noun:test".into(),
        "noun:nominative:singular:inanimate".into(),
        "градъ".into(),
        "гра́дъ".into(),
        "test-evidence".into(),
        "normative-table".into(),
        TARGET.into(),
    ];
    let duplicate = Table {
        rows: vec![row.clone(), row],
    };
    assert!(validate_exact_forms(Path::new("exact_forms.tsv"), &duplicate, &lexemes).is_err());
}

#[test]
fn one_target_token_cannot_confirm_incompatible_lexical_identities() {
    let lexical_row = |suffix: &str| {
        vec![
            format!("review:{suffix}"),
            format!("synodal:noun:{suffix}"),
            format!("sense:{suffix}"),
            "слово".into(),
            "noun".into(),
            "lexical-form".into(),
            "слово".into(),
            "сло́во".into(),
            format!("sense {suffix}"),
            "general".into(),
            "semantic-source".into(),
            format!("synodal:candidate:semantic:{suffix}"),
            "ponomar-elizabeth-bible-2026-08-09".into(),
            "synodal:candidate:shared-target".into(),
            "Passage.1".into(),
            "reviewed".into(),
            TARGET.into(),
            "contextually reviewed".into(),
        ]
    };
    let lexical = Table {
        rows: vec![lexical_row("one"), lexical_row("two")],
    };
    let ambiguities = Table { rows: vec![] };
    assert!(
        validate_lexical_reviews(Path::new("lexical_reviews.tsv"), &lexical, &ambiguities).is_err()
    );

    let exact = Table {
        rows: vec![
            vec![
                "synodal:noun:one".into(),
                "noun:nominative:singular:inanimate".into(),
                "слово".into(),
                "сло́во".into(),
                "shared-target".into(),
                "synodal-attestation".into(),
                TARGET.into(),
            ],
            vec![
                "synodal:noun:two".into(),
                "noun:nominative:singular:inanimate".into(),
                "слово".into(),
                "сло́во".into(),
                "shared-target-alias".into(),
                "synodal-attestation".into(),
                TARGET.into(),
            ],
        ],
    };
    let provenance = Table {
        rows: vec![
            vec![
                "shared-target".into(),
                "target-source".into(),
                TARGET.into(),
                "Passage.1".into(),
                "target-attestation".into(),
                "contextually reviewed".into(),
            ],
            vec![
                "shared-target-alias".into(),
                "target-source".into(),
                TARGET.into(),
                "Passage.1".into(),
                "target-attestation".into(),
                "contextually reviewed".into(),
            ],
        ],
    };
    let reviewed_evidence = Table {
        rows: vec![
            vec![
                "shared-target".into(),
                "synodal:candidate:shared-target".into(),
            ],
            vec![
                "shared-target-alias".into(),
                "synodal:candidate:shared-target".into(),
            ],
        ],
    };
    let lexical_reviews = Table { rows: vec![] };
    assert!(
        validate_exact_form_attestation_evidence(
            Path::new("exact_forms.tsv"),
            &exact,
            &provenance,
            &reviewed_evidence,
            &lexical_reviews,
            &ambiguities,
        )
        .is_err()
    );

    let adjudicated = Table {
        rows: vec![vec![
            "v07-target-shared".into(),
            "synodal:candidate:shared-target".into(),
            "слово".into(),
            "сло́во".into(),
            "synodal:noun:one".into(),
            "noun:nominative:singular:inanimate".into(),
            "synodal:noun:two".into(),
            "noun:genitive:singular:inanimate".into(),
            "adjudicated".into(),
            "the two exact cells are contextually ambiguous".into(),
        ]],
    };
    let reviewed_ambiguity_evidence = Table {
        rows: vec![vec![
            "v07-target-shared".into(),
            "synodal:candidate:shared-target".into(),
            "ponomar-elizabeth-bible-2026-08-09".into(),
            "Passage.1".into(),
            "reviewed".into(),
            TARGET.into(),
            "contextually reviewed".into(),
        ]],
    };
    validate_target_identity_ambiguities(
        Path::new("ambiguities.tsv"),
        &adjudicated,
        &reviewed_ambiguity_evidence,
    )
    .expect("valid exact-cell adjudication");
    assert!(
        validate_exact_form_attestation_evidence(
            Path::new("exact_forms.tsv"),
            &exact,
            &provenance,
            &reviewed_evidence,
            &lexical_reviews,
            &adjudicated,
        )
        .is_err(),
        "an adjudication for a different cell must not authorize this analysis"
    );
    let mut exact_adjudicated = exact.clone();
    exact_adjudicated.rows[1][1] = "noun:genitive:singular:inanimate".into();
    assert!(
        validate_exact_form_attestation_evidence(
            Path::new("exact_forms.tsv"),
            &exact_adjudicated,
            &provenance,
            &reviewed_evidence,
            &lexical_reviews,
            &adjudicated,
        )
        .is_ok(),
        "only the explicitly adjudicated cell pair is permitted"
    );

    let wrong_owner = lexical_row("one");
    let exact = Table {
        rows: vec![vec![
            "synodal:noun:two".into(),
            "lexical-form".into(),
            "слово".into(),
            "сло́во".into(),
            "review:one".into(),
            "synodal-attestation".into(),
            TARGET.into(),
        ]],
    };
    let provenance = Table {
        rows: vec![vec![
            "review:one".into(),
            "target-source".into(),
            TARGET.into(),
            "Passage.1".into(),
            "reviewed-cell:lexical-form".into(),
            "contextually reviewed".into(),
        ]],
    };
    assert!(
        validate_exact_form_attestation_evidence(
            Path::new("exact_forms.tsv"),
            &exact,
            &provenance,
            &Table { rows: vec![] },
            &Table {
                rows: vec![wrong_owner],
            },
            &ambiguities,
        )
        .is_err()
    );
}

#[test]
fn reviewed_senses_preserve_registered_source_recension() {
    let reviewed_row = |review_id: &str, source_id: &str| {
        vec![
            review_id.into(),
            format!("synodal:noun:{review_id}"),
            format!("sense:{review_id}"),
            "слово".into(),
            "noun".into(),
            "lexical-form".into(),
            "слово".into(),
            "сло́во".into(),
            "word".into(),
            "general".into(),
            source_id.into(),
            format!("synodal:candidate:{review_id}:semantic"),
            "ponomar-elizabeth-bible-2026-08-09".into(),
            format!("synodal:candidate:{review_id}:attestation"),
            "Passage.1".into(),
            "reviewed".into(),
            TARGET.into(),
            "reviewed fixture".into(),
        ]
    };
    let reviews = Table {
        rows: vec![
            reviewed_row("ocs", "ocs-source"),
            reviewed_row("mixed", "mixed-source"),
            reviewed_row("synodal", "synodal-source"),
        ],
    };
    let source_recensions = BTreeMap::from([
        ("ocs-source".into(), "old-church-slavonic".into()),
        ("mixed-source".into(), "mixed".into()),
        ("synodal-source".into(), "synodal-russian".into()),
        ("ponomar-elizabeth-bible-2026-08-09".into(), TARGET.into()),
    ]);

    let (_, _, senses) = admitted_lexical_review_rows(&reviews, &source_recensions)
        .expect("registered semantic sources");
    assert_eq!(
        senses
            .iter()
            .map(|sense| (sense[5].as_str(), sense[6].as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("old-church-slavonic", "reviewed-ocs-inheritance"),
            ("mixed", "reviewed-with-synodal-corpus"),
            ("synodal-russian", "normative"),
        ]
    );
    assert!(
        admitted_lexical_review_rows(&reviews, &BTreeMap::new()).is_err(),
        "unregistered semantic sources must fail closed"
    );
}

#[test]
fn noun_restrictions_require_a_matching_noun_recension() {
    let restrictions = Table {
        rows: vec![vec![
            "synodal:test".into(),
            "plural-only".into(),
            "animate".into(),
            "evidence:test".into(),
            TARGET.into(),
        ]],
    };
    let lexeme = |part_of_speech: &str, target: &str| Table {
        rows: vec![vec![
            "synodal:test".into(),
            "тестъ".into(),
            part_of_speech.into(),
            "exact".into(),
            String::new(),
            String::new(),
            String::new(),
            "source:test".into(),
            target.into(),
        ]],
    };

    validate_noun_restriction_lexemes(
        Path::new("noun_restrictions.tsv"),
        &restrictions,
        &lexeme("noun", TARGET),
    )
    .expect("matching noun restriction");
    assert!(
        validate_noun_restriction_lexemes(
            Path::new("noun_restrictions.tsv"),
            &restrictions,
            &lexeme("verb", TARGET),
        )
        .is_err()
    );
    assert!(
        validate_noun_restriction_lexemes(
            Path::new("noun_restrictions.tsv"),
            &restrictions,
            &lexeme("noun", "old-church-slavonic"),
        )
        .is_err()
    );

    let exact_row = |cell: &str| {
        vec![
            "synodal:test".into(),
            cell.into(),
            "тестъ".into(),
            "те́стъ".into(),
            "evidence:test".into(),
            "synodal-attestation".into(),
            TARGET.into(),
        ]
    };
    validate_noun_restriction_exact_forms(
        Path::new("noun_restrictions.tsv"),
        &restrictions,
        Path::new("exact_forms.tsv"),
        &Table {
            rows: vec![exact_row("noun:nominative:plural:animate")],
        },
    )
    .expect("exact cell inside both noun inventories");
    for incompatible in [
        "noun:nominative:singular:animate",
        "noun:nominative:plural:inanimate",
        "noun:nominative:plural:any",
        "lexical-form",
    ] {
        assert!(
            validate_noun_restriction_exact_forms(
                Path::new("noun_restrictions.tsv"),
                &restrictions,
                Path::new("exact_forms.tsv"),
                &Table {
                    rows: vec![exact_row(incompatible)],
                },
            )
            .is_err(),
            "accepted incompatible exact noun cell {incompatible}"
        );
    }
}

#[test]
fn finite_past_audit_is_exhaustive_locked_and_leaves_no_past_cells() {
    let historical = Table {
        rows: vec![vec![
            "v06-exact-03a1ca3817d4918e".into(),
            "admitted".into(),
            "source-typed-exact".into(),
            "family:test".into(),
            "synodal:verb:test".into(),
            "избити".into(),
            "verb".into(),
            "и҆збѝ".into(),
            "1".into(),
            "past:third:singular".into(),
            "semantic:test".into(),
            "morphology:test".into(),
            "target:test".into(),
            "candidate:test".into(),
            "source passage".into(),
            "evaluation passage".into(),
            "1".into(),
            "1".into(),
            String::new(),
            "historical review".into(),
        ]],
    };
    let reviews = Table {
        rows: vec![vec![
            "v06-exact-03a1ca3817d4918e".into(),
            "synodal:verb:test".into(),
            "избити".into(),
            "past:third:singular".into(),
            "и҆збѝ".into(),
            "reclassified-aorist".into(),
            "aorist:third:singular".into(),
            "source passage".into(),
            "evaluation passage".into(),
            "reviewed against the aorist grammar".into(),
        ]],
    };
    let exact = Table {
        rows: vec![vec![
            "synodal:verb:test".into(),
            "aorist:third:singular".into(),
            "изби".into(),
            "и҆збѝ".into(),
            "evidence:test".into(),
            "synodal-attestation".into(),
            TARGET.into(),
        ]],
    };
    let evaluation = Table {
        rows: vec![vec![
            "eval:test".into(),
            "synodal:verb:test".into(),
            "aorist:third:singular".into(),
            "strict".into(),
            "изби".into(),
            "и҆збѝ".into(),
            "source:test".into(),
            "evaluation passage".into(),
            "fixture".into(),
        ]],
    };
    let validate = |reviews: &Table, evaluation: &Table| {
        validate_past_classification_reviews(
            (Path::new("past_classification_reviews.tsv"), reviews),
            (Path::new("v06_exact_reviews.tsv"), &historical),
            (Path::new("exact_forms.tsv"), &exact),
            (Path::new("evaluation.tsv"), evaluation),
        )
    };

    validate(&reviews, &evaluation).expect("complete reclassification audit");
    assert!(validate(&Table { rows: Vec::new() }, &evaluation).is_err());

    let mut altered = reviews.clone();
    altered.rows[0][5] = "reclassified-imperfect".into();
    altered.rows[0][6] = "imperfect:third:singular".into();
    assert!(validate(&altered, &evaluation).is_err());

    let mut surviving_past = evaluation.clone();
    surviving_past.rows[0][2] = "past:third:singular".into();
    assert!(validate(&reviews, &surviving_past).is_err());
}

#[test]
fn target_registry_rejects_the_historically_merged_supine_category() {
    let exact = Table { rows: Vec::new() };
    let evaluation = Table { rows: Vec::new() };
    let validate = |exact: &Table, evaluation: &Table| {
        validate_absent_target_cells(
            (Path::new("exact_forms.tsv"), exact),
            (Path::new("evaluation.tsv"), evaluation),
        )
    };
    validate(&exact, &evaluation).expect("empty target supine inventory");

    let exact_supine = Table {
        rows: vec![vec![
            "synodal:verb:test".into(),
            "supine".into(),
            "нестъ".into(),
            "не́стъ".into(),
            "evidence:test".into(),
            "synodal-attestation".into(),
            TARGET.into(),
        ]],
    };
    assert!(validate(&exact_supine, &evaluation).is_err());

    let evaluation_supine = Table {
        rows: vec![vec![
            "eval:test".into(),
            "synodal:verb:test".into(),
            "supine".into(),
            "strict".into(),
            "нестъ".into(),
            "не́стъ".into(),
            "source:test".into(),
            "passage".into(),
            "fixture".into(),
        ]],
    };
    assert!(validate(&exact, &evaluation_supine).is_err());
}

#[test]
fn defective_inventories_are_closed_typed_and_verb_only() {
    let path = Path::new("verb_defectiveness.tsv");
    let verb = vec![
        "synodal:verb:test".into(),
        "подобати".into(),
        "verb".into(),
        "exact".into(),
        String::new(),
        String::new(),
        String::new(),
        "source".into(),
        TARGET.into(),
    ];
    let noun = vec![
        "synodal:noun:test".into(),
        "слово".into(),
        "noun".into(),
        "exact".into(),
        String::new(),
        "neuter".into(),
        String::new(),
        "source".into(),
        TARGET.into(),
    ];
    let lexemes = Table {
        rows: vec![verb, noun],
    };
    let valid = Table {
        rows: vec![vec![
            "synodal:verb:test".into(),
            "outside-inventory".into(),
            "infinitive,present:third:singular".into(),
            "historically-absent".into(),
            "irregular-override".into(),
            "closed impersonal inventory".into(),
            "evidence:test".into(),
            TARGET.into(),
        ]],
    };
    validate_defective_inventories(path, &valid, &lexemes).expect("valid typed defect inventory");

    let mutate = |column: usize, value: &str| {
        let mut table = valid.clone();
        table.rows[0][column] = value.into();
        table
    };
    assert!(validate_defective_inventories(path, &mutate(1, "unknown"), &lexemes).is_err());
    assert!(
        validate_defective_inventories(path, &mutate(2, "present:fourth:singular"), &lexemes)
            .is_err()
    );
    assert!(validate_defective_inventories(path, &mutate(3, "unknown"), &lexemes).is_err());
    assert!(validate_defective_inventories(path, &mutate(4, "untyped-field"), &lexemes).is_err());
    assert!(
        validate_defective_inventories(path, &mutate(0, "synodal:noun:test"), &lexemes).is_err()
    );

    let prefix = Table {
        rows: vec![vec![
            "synodal:verb:test".into(),
            "cell-prefix".into(),
            "participle:present:passive:".into(),
            "historically-absent".into(),
            "participle-formation".into(),
            "explicitly absent system".into(),
            "evidence:test".into(),
            TARGET.into(),
        ]],
    };
    validate_defective_inventories(path, &prefix, &lexemes).expect("valid system-level defect");
}

#[test]
fn irregular_inventory_requires_all_98_source_order_entries() {
    let row = |order: u8| {
        vec![
            order.to_string(),
            format!("headword-{order}"),
            "present".into(),
            "caller-exact-principal-parts".into(),
            "implemented-by-metadata-contract".into(),
            "evidence:test".into(),
            "reviewed source entry".into(),
            TARGET.into(),
        ]
    };
    let complete = Table {
        rows: (2_u8..=100).filter(|order| *order != 97).map(row).collect(),
    };
    let path = Path::new("irregular_verb_inventory.tsv");
    validate_irregular_verb_inventory(path, &complete).expect("complete §104 inventory");

    let mut missing = complete.clone();
    missing.rows.retain(|row| row[0] != "55");
    assert!(validate_irregular_verb_inventory(path, &missing).is_err());

    let mut unknown_system = complete.clone();
    unknown_system.rows[0][2] = "invented-system".into();
    assert!(validate_irregular_verb_inventory(path, &unknown_system).is_err());

    let mut unknown_strategy = complete;
    unknown_strategy.rows[0][3] = "guess".into();
    assert!(validate_irregular_verb_inventory(path, &unknown_strategy).is_err());
}
