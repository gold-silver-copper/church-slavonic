//! Bounded, staged corpus extraction. Cache contents are checked against the
//! actual archive on every call; a digest or editable manifest alone is not trust.
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::{Component, Path, PathBuf},
};
const COMPRESSED: u64 = 64 * 1024 * 1024;
const EXPANDED: u64 = 512 * 1024 * 1024;
const FILE_SIZE: u64 = 64 * 1024 * 1024;
const ENTRIES: usize = 10_000;
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn hash(reader: impl Read) -> io::Result<(u64, String)> {
    let mut reader = reader;
    let mut digest = Sha256::new();
    let mut size = 0;
    let mut buffer = [0; 32768];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
        size += n as u64;
    }
    Ok((size, format!("{:x}", digest.finalize())))
}
fn inventory(root: &Path) -> io::Result<BTreeMap<PathBuf, (u64, String)>> {
    if !fs::symlink_metadata(root)?.file_type().is_dir() {
        return Err(invalid("cache root is not a directory"));
    }
    let mut pending = vec![root.to_path_buf()];
    let mut out = BTreeMap::new();
    let mut count = 0;
    let mut bytes = 0;
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            count += 1;
            if count > ENTRIES {
                return Err(invalid("cache entry budget"));
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                let record = hash(File::open(entry.path())?.take(FILE_SIZE + 1))?;
                bytes += record.0;
                if record.0 > FILE_SIZE || bytes > EXPANDED {
                    return Err(invalid("cache byte budget"));
                }
                out.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .map_err(|e| invalid(e.to_string()))?
                        .to_path_buf(),
                    record,
                );
            } else {
                return Err(invalid("cache links and special files are forbidden"));
            }
        }
    }
    Ok(out)
}

pub fn verified_extract(source: &Path, cache: &Path) -> io::Result<PathBuf> {
    if !fs::symlink_metadata(source)?.file_type().is_file() {
        return Err(invalid("source is not a regular file"));
    }
    fs::create_dir_all(cache)?;
    let work = tempfile::Builder::new()
        .prefix(".stage-")
        .tempdir_in(cache)?;
    // Copy to a private snapshot before hashing/parsing, so a source replacement
    // cannot make the cache key describe a different read of the archive.
    let mut snapshot = tempfile::tempfile_in(work.path())?;
    let copied = io::copy(&mut File::open(source)?.take(COMPRESSED + 1), &mut snapshot)?;
    if copied > COMPRESSED {
        return Err(invalid("compressed archive byte budget"));
    }
    snapshot.rewind()?;
    let (_, digest) = hash(&mut snapshot)?;
    snapshot.rewind()?;
    let target = cache.join(digest);
    let reuse = match fs::symlink_metadata(&target) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(e),
    };
    let payload = work.path().join("payload");
    fs::create_dir(&payload)?;
    let mut archive =
        tar::Archive::new(flate2::read::MultiGzDecoder::new(snapshot).take(EXPANDED + 1));
    let mut names = std::collections::BTreeSet::new();
    let mut expected = BTreeMap::new();
    let mut filesystem_paths = std::collections::BTreeSet::new();
    for (index, entry) in archive.entries()?.enumerate() {
        if index >= ENTRIES {
            return Err(invalid("archive entry budget"));
        }
        let mut entry = entry?;
        if entry.header().entry_type().is_pax_global_extensions() {
            if entry.size() > 16 * 1024 {
                return Err(invalid("global metadata byte budget"));
            }
            if let Some(extensions) = entry.pax_extensions()? {
                for extension in extensions {
                    if extension?.key_bytes() != b"comment" {
                        return Err(invalid("unsupported global archive metadata"));
                    }
                }
            }
            continue;
        }
        let path = entry.path()?.into_owned();
        if path.as_os_str().len() > 4096
            || path.components().count() > 64
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || path.as_os_str().is_empty()
        {
            return Err(invalid("unsafe archive path"));
        }
        if !names.insert(path.clone()) {
            return Err(invalid("duplicate archive path"));
        }
        if names.len() > ENTRIES {
            return Err(invalid("archive entry budget"));
        }
        for ancestor in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
            filesystem_paths.insert(ancestor.to_path_buf());
            if filesystem_paths.len() > ENTRIES {
                return Err(invalid("expanded filesystem entry budget"));
            }
        }
        let dest = payload.join(&path);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            if !reuse {
                fs::create_dir_all(dest)?;
            }
        } else if kind.is_file() {
            if entry.size() > FILE_SIZE {
                return Err(invalid("archive file byte budget"));
            }
            let declared = entry.size();
            let record = if reuse {
                hash(&mut entry)?
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = File::options().write(true).create_new(true).open(&dest)?;
                io::copy(&mut entry, &mut file)?;
                file.flush()?;
                hash(File::open(dest)?)?
            };
            if record.0 != declared {
                return Err(invalid("truncated archive file"));
            }
            expected.insert(path, record);
        } else {
            return Err(invalid("archive links and special files are forbidden"));
        }
    }
    // Consume the stream for gzip integrity and expansion-budget validation;
    // nonzero material after the tar terminator is not silently accepted.
    let mut remaining = archive.into_inner();
    let mut buffer = [0; 32768];
    loop {
        let n = remaining.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        if buffer[..n].iter().any(|b| *b != 0) {
            return Err(invalid("trailing archive data"));
        }
    }
    if remaining.limit() == 0 {
        return Err(invalid("expanded archive byte budget"));
    }
    if expected.is_empty() {
        return Err(invalid("archive contains no files"));
    }
    if reuse {
        if inventory(&target)? != expected {
            return Err(invalid(format!(
                "cache does not match archive: {}",
                target.display()
            )));
        }
    } else {
        // Rename within one filesystem: consumers see a complete directory.
        // Another writer may have published the same digest concurrently.
        if let Err(e) = fs::rename(&payload, &target)
            && (!target.is_dir() || inventory(&target)? != expected)
        {
            return Err(e);
        }
    }
    Ok(target)
}
