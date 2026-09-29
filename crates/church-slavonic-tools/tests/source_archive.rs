#![allow(clippy::unwrap_used)]
use church_slavonic_tools::sources::{archive::verified_extract, ud};
use std::{fs, io::Write, path::Path};
fn archive(path: &Path, rows: &[(&str, &[u8], u8)]) {
    let encoder =
        flate2::write::GzEncoder::new(fs::File::create(path).unwrap(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (name, bytes, kind) in rows {
        let mut header = tar::Header::new_gnu();
        // Direct header bytes allow deliberately invalid paths in controls.
        header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::new(*kind));
        header.set_cksum();
        builder.append(&header, *bytes).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap();
}
#[test]
fn verified_reuse_rejects_changed_missing_extra_and_linked_cache_files() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.tar.gz");
    let cache = tmp.path().join("cache");
    archive(&source, &[("root/a.txt", b"observed", b'0')]);
    let first = verified_extract(&source, &cache).unwrap();
    assert_eq!(verified_extract(&source, &cache).unwrap(), first);
    fs::write(first.join("root/a.txt"), "tampered").unwrap();
    assert!(verified_extract(&source, &cache).is_err());
    fs::write(first.join("root/a.txt"), "observed").unwrap();
    fs::write(first.join("extra"), "extra").unwrap();
    assert!(verified_extract(&source, &cache).is_err());
    fs::remove_file(first.join("extra")).unwrap();
    fs::remove_file(first.join("root/a.txt")).unwrap();
    assert!(verified_extract(&source, &cache).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&source, first.join("root/a.txt")).unwrap();
        assert!(verified_extract(&source, &cache).is_err());
    }
}
#[test]
fn changed_source_gets_new_snapshot_and_failed_extraction_is_not_published() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.tar.gz");
    let cache = tmp.path().join("cache");
    archive(&source, &[("a", b"old", b'0')]);
    let old = verified_extract(&source, &cache).unwrap();
    archive(&source, &[("a", b"new", b'0')]);
    let new = verified_extract(&source, &cache).unwrap();
    assert_ne!(old, new);
    assert_eq!(fs::read(old.join("a")).unwrap(), b"old");
    assert_eq!(fs::read(new.join("a")).unwrap(), b"new");
    for rows in [
        vec![("../escape", &b"bad"[..], b'0')],
        vec![("link", &b""[..], b'2')],
        vec![("a", &b"x"[..], b'0'), ("a", &b"y"[..], b'0')],
    ] {
        archive(&source, &rows);
        assert!(verified_extract(&source, &cache).is_err());
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 2);
    }
    assert!(!tmp.path().join("escape").exists());
}
#[test]
fn corrupt_gzip_and_empty_archives_fail_without_a_cache_result() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.tar.gz");
    let cache = tmp.path().join("cache");
    archive(&source, &[]);
    assert!(verified_extract(&source, &cache).is_err());
    archive(&source, &[("a", b"hello", b'0')]);
    let mut bytes = fs::read(&source).unwrap();
    bytes.truncate(bytes.len() - 5);
    fs::write(&source, bytes).unwrap();
    assert!(verified_extract(&source, &cache).is_err());
    assert_eq!(fs::read_dir(cache).unwrap().count(), 0);
}
#[test]
fn corpus_loader_uses_verified_cache_and_rejects_ambiguous_source_selection() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join(ud::UD_PROIEL_SOURCE);
    fs::create_dir(&source).unwrap();
    let artifacts = tmp.path().join("artifacts");
    assert!(ud::load_ud_proiel_heldout(tmp.path(), &artifacts).is_err());
    let conllu = "# sent_id = fixture\n1\tградъ\tградъ\tNOUN\t_\tCase=Nom|Number=Sing|Gender=Masc\t0\troot\t_\t_\n\n";
    archive(
        &source.join("one.tar.gz"),
        &[("root/cu_proiel-ud-dev.conllu", conllu.as_bytes(), b'0')],
    );
    let corpus = ud::load_ud_proiel_heldout(tmp.path(), &artifacts)
        .unwrap()
        .unwrap();
    assert_eq!(corpus.tokens, 1);
    assert_eq!(corpus.sentences[0][0].surface, "градъ");
    fs::File::create(source.join("two.tar.gz"))
        .unwrap()
        .write_all(b"bad")
        .unwrap();
    assert!(ud::load_ud_proiel_heldout(tmp.path(), &artifacts).is_err());
}

#[test]
fn declared_oversized_file_is_rejected_before_extraction() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.tar.gz");
    let mut header = tar::Header::new_gnu();
    header.set_path("huge").unwrap();
    header.set_size(64 * 1024 * 1024 + 1);
    header.set_mode(0o644);
    header.set_cksum();
    let mut encoder = flate2::write::GzEncoder::new(
        fs::File::create(&source).unwrap(),
        flate2::Compression::fast(),
    );
    encoder.write_all(header.as_bytes()).unwrap();
    encoder.finish().unwrap();
    let cache = tmp.path().join("cache");
    assert!(
        verified_extract(&source, &cache)
            .unwrap_err()
            .to_string()
            .contains("file byte budget")
    );
    assert_eq!(fs::read_dir(cache).unwrap().count(), 0);
}

#[test]
#[ignore = "requires the downloaded local corpus archives; run explicitly"]
fn downloaded_corpus_archives_use_verified_cache() {
    let root = church_slavonic_tools::workspace_root();
    let sources = root.join("references/downloads");
    let artifacts = root.join("target/sources");
    let ud = ud::load_ud_proiel_heldout(&sources, &artifacts)
        .unwrap()
        .unwrap();
    assert_eq!(
        ud.tokens as usize,
        ud.sentences.iter().map(Vec::len).sum::<usize>()
    );
    println!("UD dev/test tokens: {}", ud.tokens);
    let synt = ud::load_syntacticus(&sources, &artifacts).unwrap().unwrap();
    assert_eq!(
        synt.tokens as usize,
        synt.sentences.iter().map(Vec::len).sum::<usize>()
    );
    println!("Syntacticus tokens: {}", synt.tokens);
}

#[test]
fn git_archive_comment_metadata_is_supported_but_global_path_changes_are_not() {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.tar.gz");
    let cache = tmp.path().join("cache");
    archive(
        &source,
        &[
            ("pax_global_header", b"13 comment=x\n", b'g'),
            ("a", b"text", b'0'),
        ],
    );
    let result = verified_extract(&source, &cache).unwrap();
    assert_eq!(fs::read(result.join("a")).unwrap(), b"text");
    assert!(!result.join("pax_global_header").exists());
    archive(
        &source,
        &[
            ("pax_global_header", b"15 path=escape\n", b'g'),
            ("a", b"text", b'0'),
        ],
    );
    assert!(verified_extract(&source, &cache).is_err());
}
