use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

pub(crate) struct Table {
    path: PathBuf,
    pub(crate) header: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

impl Table {
    pub(crate) fn index(&self, name: &str) -> Result<usize, Box<dyn Error>> {
        self.header
            .iter()
            .position(|column| column == name)
            .ok_or_else(|| format!("{} omits column {name:?}", self.path.display()).into())
    }
}

pub(crate) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn read_tsv(path: &Path) -> Result<Table, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| format!("{} is empty", path.display()))?
        .split('\t')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let rows = lines
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(offset, line)| {
            let row = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
            if row.len() == header.len() {
                Ok(row)
            } else {
                Err(format!(
                    "{}:{} has {} fields; expected {}",
                    path.display(),
                    offset + 2,
                    row.len(),
                    header.len()
                ))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Table {
        path: path.into(),
        header,
        rows,
    })
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
        let json = concat!(
            "{\"value\":2,\"kind\":\"audit\",\"nested\":{\"count\":3},",
            "\"normalized_tables\":{\"forms\":4}}\n"
        );
        write_if_changed_atomic(&path, json).expect("atomic update");
        check_contents_for(&path, json, "example-command").expect("current report");

        let value: serde_json::Value = read_json(&path).expect("valid JSON");
        assert_eq!(value["value"], 2);
        assert_eq!(root_number(&value, "value").expect("root number"), 2);
        assert_eq!(string(&value, "kind").expect("root string"), "audit");
        require_string(&value, "kind", "audit").expect("matching string");
        require_number(&value, "/nested/count", 3).expect("matching number");
        assert_eq!(pointer_number(&value, "/nested/count").expect("pointer"), 3);
        assert_eq!(table_count(&value, "forms").expect("table count"), 4);
        let nested = object(&value, "nested").expect("nested object");
        assert_eq!(number(nested, "count").expect("nested number"), 3);
        let normalized = value["normalized_tables"]
            .as_object()
            .expect("normalized tables object");
        let mapped_value = serde_json::json!({ "outer": { "count": 5 } });
        let mapped_root = mapped_value.as_object().expect("outer object");
        let mapped = map_object(mapped_root, "outer").expect("mapped object");
        assert_eq!(number(mapped, "count").expect("mapped number"), 5);
        assert_eq!(number(normalized, "forms").expect("normalized count"), 4);
        assert_eq!(percent(1, 8), "12.500%");
        assert_eq!(escape("a|b\nc"), "a\\|b c");

        let table_path = root.join("review.tsv");
        fs::write(&table_path, "lemma\tstatus\nслово\treviewed\n").expect("temporary TSV");
        let table = read_tsv(&table_path).expect("valid TSV");
        require_header(&table, &["lemma", "status"]).expect("required columns");
        assert_eq!(table.index("status").expect("status column"), 1);
        assert_eq!(table.rows, vec![vec!["слово", "reviewed"]]);
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
