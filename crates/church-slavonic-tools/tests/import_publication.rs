#![allow(clippy::unwrap_used)]
use church_slavonic::{Pos, Recension, lexicon};
use church_slavonic_tools::import::{
    publication::{StagedReplacement, stage_lexicon, stage_quarantine},
    refit,
};
use std::{fs, process::Command};

const ROW: &str = "audit.n\tрабъ\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ\t-\t-\t-\n";

#[test]
fn actual_refit_consumer_replaces_only_a_complete_validated_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nouns.tsv");
    fs::write(&path, ROW).unwrap();
    refit::run_file(&path, Pos::Noun, false).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), ROW);
    refit::run_file(&path, Pos::Noun, true).unwrap();
    let published = fs::read_to_string(&path).unwrap();
    assert_eq!(lexicon::parse(&published, Pos::Noun).unwrap().len(), 1);
    assert_eq!(
        published,
        lexicon::format(&lexicon::parse(ROW, Pos::Noun).unwrap())
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn rejected_lexemes_and_quarantine_preserve_destinations() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nouns.tsv");
    fs::write(&path, ROW).unwrap();
    let rows = lexicon::parse(ROW, Pos::Noun).unwrap();
    for bad in 0..3 {
        let mut changed = rows.clone();
        match bad {
            0 => changed.push(changed[0].clone()),
            1 => changed[0].recension = Recension::OldChurchSlavonic,
            _ => changed[0].stress = "not-a-stress-spec".into(),
        }
        assert!(stage_lexicon(&path, &changed, Pos::Noun, Recension::Synodal).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), ROW);
    }
    let stage = stage_lexicon(&path, &rows, Pos::Noun, Recension::Synodal).unwrap();
    let qpath = dir.path().join("quarantine.tsv");
    fs::write(&qpath, "old quarantine").unwrap();
    assert!(stage_quarantine(&qpath, "syn\tn\tbroken").is_err());
    drop(stage);
    assert_eq!(fs::read_to_string(&path).unwrap(), ROW);
    assert_eq!(fs::read_to_string(&qpath).unwrap(), "old quarantine");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn validation_drop_and_rename_failure_never_publish_partial_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("output");
    fs::write(&path, b"old").unwrap();
    assert!(StagedReplacement::prepare(&path, b"bad", |_| Err("rejected".into())).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"old");
    let staged = StagedReplacement::prepare(&path, b"complete", |b| {
        assert_eq!(b, b"complete");
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"old");
    drop(staged);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    let absent = dir.path().join("absent");
    let staged = StagedReplacement::prepare(&absent, b"complete", |_| Ok(())).unwrap();
    fs::create_dir(&absent).unwrap();
    fs::write(absent.join("marker"), "untouched").unwrap();
    assert!(staged.commit().is_err());
    assert_eq!(
        fs::read_to_string(absent.join("marker")).unwrap(),
        "untouched"
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn process_exit_after_staging_does_not_replace_live_file() {
    const ENV: &str = "SLAVONIC_STAGED_EXIT_TEST_PATH";
    if let Some(path) = std::env::var_os(ENV) {
        let _stage =
            StagedReplacement::prepare(std::path::Path::new(&path), b"new complete bytes", |_| {
                Ok(())
            })
            .unwrap();
        // Bypass destructors as an interrupted importer would. No live rename.
        std::process::exit(42);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live");
    fs::write(&path, "original bytes").unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process_exit_after_staging_does_not_replace_live_file",
        ])
        .env(ENV, &path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(42));
    assert_eq!(fs::read_to_string(&path).unwrap(), "original bytes");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2); // unpublished sibling remains
}

#[cfg(unix)]
#[test]
fn replacement_keeps_mode_and_refuses_symlink_destinations() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live");
    fs::write(&path, "old").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    StagedReplacement::prepare(&path, b"new", |_| Ok(()))
        .unwrap()
        .commit()
        .unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
    let link = dir.path().join("link");
    symlink(&path, &link).unwrap();
    assert!(StagedReplacement::prepare(&link, b"overwrite", |_| Ok(())).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
}

#[test]
fn current_lexicon_files_can_be_validated_without_publishing_them() {
    let dir = tempfile::tempdir().unwrap();
    for (folder, recension) in [
        ("syn", Recension::Synodal),
        ("ocs", Recension::OldChurchSlavonic),
    ] {
        for pos in [
            Pos::Noun,
            Pos::Adjective,
            Pos::Verb,
            Pos::Pronoun,
            Pos::Closed,
        ] {
            // The current OCS lexicon has no separate closed.tsv asset.
            if recension == Recension::OldChurchSlavonic && pos == Pos::Closed {
                continue;
            }
            let filename = church_slavonic_tools::import::lexicon_file(pos);
            let source = church_slavonic_tools::import::lexicon_dir()
                .join(folder)
                .join(filename);
            let bytes = fs::read(&source).unwrap();
            let rows =
                lexicon::parse_in(std::str::from_utf8(&bytes).unwrap(), pos, recension).unwrap();
            let destination = dir.path().join(format!("{folder}-{filename}"));
            let stage = stage_lexicon(&destination, &rows, pos, recension).unwrap();
            drop(stage);
            assert!(!destination.exists());
            assert_eq!(fs::read(&source).unwrap(), bytes);
        }
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn importer_checks_both_outputs_and_duplicate_inputs_before_publication() {
    use church_slavonic_tools::import::{Outcome, Quarantined, write_outcome_at};
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("syn")).unwrap();
    let path = dir.path().join("syn/nouns.tsv");
    fs::write(&path, ROW).unwrap();
    let qpath = dir.path().join("quarantine.tsv");
    let mut outcome = Outcome {
        lexemes: lexicon::parse(ROW, Pos::Noun).unwrap(),
        ..Outcome::default()
    };
    // A read error is not an absent previous quarantine.
    fs::create_dir(&qpath).unwrap();
    assert!(write_outcome_at(dir.path(), &outcome, Recension::Synodal, Pos::Noun, "P:").is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), ROW);
    fs::remove_dir(&qpath).unwrap();
    fs::write(&qpath, "# original\n").unwrap();
    outcome.lexemes.push(outcome.lexemes[0].clone());
    assert!(write_outcome_at(dir.path(), &outcome, Recension::Synodal, Pos::Noun, "P:").is_err());
    outcome.lexemes.pop();
    outcome.quarantine.push(Quarantined {
        recension: Recension::Synodal,
        pos: Pos::Noun,
        lemma: "bad\nrow".into(),
        source: "test".into(),
        reason: "constructed",
        detail: String::new(),
    });
    assert!(write_outcome_at(dir.path(), &outcome, Recension::Synodal, Pos::Noun, "P:").is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), ROW);
    assert_eq!(fs::read_to_string(&qpath).unwrap(), "# original\n");
    outcome.quarantine[0].lemma = "test".into();
    write_outcome_at(dir.path(), &outcome, Recension::Synodal, Pos::Noun, "P:").unwrap();
    assert_eq!(
        lexicon::parse(&fs::read_to_string(&path).unwrap(), Pos::Noun)
            .unwrap()
            .len(),
        1
    );
    assert!(
        fs::read_to_string(&qpath)
            .unwrap()
            .contains("syn\tn\ttest\ttest\tconstructed\t-\n")
    );
    assert_eq!(fs::read_dir(dir.path().join("syn")).unwrap().count(), 1);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}
