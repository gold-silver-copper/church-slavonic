use std::{error::Error, fs, path::Path};

use serde::de::DeserializeOwned;

pub(crate) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn check_contents(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(expected) {
        Ok(())
    } else {
        Err(format!("{} is stale", path.display()).into())
    }
}

pub(crate) fn check_contents_for(
    path: &Path,
    expected: &str,
    command: &str,
) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(expected) {
        Ok(())
    } else {
        Err(format!("stale {}; rerun {command}", path.display()).into())
    }
}

pub(crate) fn write_if_changed(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    fs::write(path, contents)?;
    Ok(())
}

pub(crate) fn write_if_changed_atomic(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("new")
    ));
    fs::write(&temporary, contents)?;
    fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_report_io_preserves_write_and_stale_contracts() {
        let root = std::env::temp_dir().join(format!("xtask-report-io-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temporary directory");
        let path = root.join("report.json");

        write_if_changed(&path, "{\"value\":1}\n").expect("direct write");
        check_contents(&path, "{\"value\":1}\n").expect("current report");
        write_if_changed_atomic(&path, "{\"value\":2}\n").expect("atomic update");
        check_contents_for(&path, "{\"value\":2}\n", "example-command").expect("current report");

        let value: serde_json::Value = read_json(&path).expect("valid JSON");
        assert_eq!(value["value"], 2);
        assert_eq!(
            check_contents(&path, "different")
                .expect_err("stale report")
                .to_string(),
            format!("{} is stale", path.display())
        );
        assert_eq!(
            check_contents_for(&path, "different", "example-command")
                .expect_err("stale report")
                .to_string(),
            format!("stale {}; rerun example-command", path.display())
        );

        fs::remove_dir_all(root).expect("temporary cleanup");
    }
}
