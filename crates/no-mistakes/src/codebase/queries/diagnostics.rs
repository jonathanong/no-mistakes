use super::shared::rel_str;
use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
use serde::Serialize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Collection gaps in the requested reverse scan, not a soundness guarantee.
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QueryDiagnostics {
    partial: bool,
    incomplete_files: Vec<QueryIncompleteFile>,
}

#[derive(Serialize)]
struct QueryIncompleteFile {
    file: String,
    reason: &'static str,
    message: String,
}

impl QueryDiagnostics {
    pub(crate) fn from_prepared(files: &[PathBuf], facts: &TsFactMap, root: &Path) -> Self {
        let mut incomplete_files: Vec<_> = files
            .iter()
            .filter_map(|file| {
                let (reason, message) = incomplete_reason(facts.get(file))?;
                Some(QueryIncompleteFile {
                    file: rel_str(file, root),
                    reason,
                    message,
                })
            })
            .collect();
        incomplete_files.sort_by(|a, b| a.file.cmp(&b.file));
        incomplete_files.dedup_by(|a, b| a.file == b.file);
        Self {
            partial: !incomplete_files.is_empty(),
            incomplete_files,
        }
    }

    pub(crate) fn is_partial(&self) -> bool {
        self.partial
    }

    pub(crate) fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        if self.partial {
            writeln!(w, "Partial analysis: references may be missing.")?;
            for file in &self.incomplete_files {
                writeln!(w, "  {} [{}]: {}", file.file, file.reason, file.message)?;
            }
        }
        Ok(())
    }
}

fn incomplete_reason(facts: Option<&TsFileFacts>) -> Option<(&'static str, String)> {
    let Some(facts) = facts else {
        return Some((
            "missing-facts",
            "Prepared source facts are unavailable".into(),
        ));
    };
    if let Some(error) = &facts.operational_error {
        return Some(("operational-error", error.clone()));
    }
    if let Some(error) = &facts.parse_error {
        return Some(("parse-error", error.clone()));
    }
    if facts.fatal_parse_error {
        return Some(("parse-error", "Fatal parser failure".into()));
    }
    if facts.symbols.is_none() {
        return Some((
            "missing-symbols",
            "Prepared symbol facts are unavailable".into(),
        ));
    }
    None
}

#[cfg(test)]
mod tests;
