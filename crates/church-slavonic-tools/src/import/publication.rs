//! Validated, atomic replacement of one local import output.
//! Multiple commits are not a transaction. Parent directories must be trusted;
//! concurrent writers and power-loss durability are outside this contract.
use church_slavonic::{
    Lexicon, Pos, Recension,
    lexicon::{self, Lexeme},
};
use std::{
    error::Error,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

/// No writable handle or temporary pathname is exposed after validation.
pub struct StagedReplacement {
    file: NamedTempFile,
    destination: PathBuf,
}
impl StagedReplacement {
    pub fn prepare(
        destination: &Path,
        bytes: &[u8],
        validate: impl FnOnce(&[u8]) -> Result<(), Box<dyn Error>>,
    ) -> Result<Self, Box<dyn Error>> {
        if bytes.len() > MAX_OUTPUT_BYTES {
            return Err("import output exceeds byte limit".into());
        }
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let permissions = match fs::symlink_metadata(destination) {
            Ok(meta) if meta.file_type().is_file() => Some(meta.permissions()),
            Ok(_) => return Err("import destination must be a regular file or absent".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let mut file = tempfile::Builder::new()
            .prefix(".slavonic-import-")
            .tempfile_in(parent)?;
        file.write_all(bytes)?;
        file.seek(SeekFrom::Start(0))?;
        let mut staged = Vec::new();
        file.as_file_mut()
            .take((MAX_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut staged)?;
        if staged != bytes {
            return Err("staged import differs from supplied bytes".into());
        }
        validate(&staged)?;
        if let Some(permissions) = permissions {
            file.as_file().set_permissions(permissions)?;
        }
        file.as_file().sync_all()?;
        Ok(Self {
            file,
            destination: destination.into(),
        })
    }

    /// Rename a validated sibling into place. Dropping an uncommitted stage
    /// cleans it up; abrupt process exit may leave an unpublished sibling.
    pub fn commit(self) -> Result<(), Box<dyn Error>> {
        self.file.persist(self.destination).map_err(|e| e.error)?;
        Ok(())
    }
}

pub fn stage_lexicon(
    path: &Path,
    entries: &[Lexeme],
    pos: Pos,
    recension: Recension,
) -> Result<StagedReplacement, Box<dyn Error>> {
    if entries.iter().any(|l| l.pos != pos) {
        return Err("mixed POS in import output".into());
    }
    Lexicon::try_from_lexemes(recension, entries.to_vec())?;
    let text = lexicon::format(entries);
    StagedReplacement::prepare(path, text.as_bytes(), |bytes| {
        let text = std::str::from_utf8(bytes)?;
        let parsed = lexicon::parse_in(text, pos, recension)?;
        if parsed.len() != entries.len()
            || parsed.iter().zip(entries).any(|(a, b)| a.id != b.id)
            || lexicon::format(&parsed) != text
        {
            return Err("lexicon output does not survive serialization".into());
        }
        Lexicon::try_from_lexemes(recension, parsed)?;
        Ok(())
    })
}

pub fn stage_quarantine(path: &Path, text: &str) -> Result<StagedReplacement, Box<dyn Error>> {
    StagedReplacement::prepare(path, text.as_bytes(), |bytes| {
        for line in std::str::from_utf8(bytes)?
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let cols: Vec<_> = line.split('\t').collect();
            if cols.len() != 6
                || !matches!(cols[0], "syn" | "ocs")
                || Pos::parse(cols[1]).is_none()
                || cols.iter().any(|c| c.is_empty() || c.contains('\r'))
            {
                return Err("invalid quarantine row".into());
            }
        }
        Ok(())
    })
}
