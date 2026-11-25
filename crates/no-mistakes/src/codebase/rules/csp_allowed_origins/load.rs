use super::{CspPolicySet, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn load_policy(
    root: &Path,
    csp_files: &[PathBuf],
    facts: Option<&CheckFactMap>,
    sources: &SourceStore,
) -> Result<CspPolicySet> {
    let mut combined = CspPolicySet::default();
    for path in csp_files {
        let file = relative_slash_path(root, path);
        let parsed = if crate::codebase::dependencies::extract::is_indexable(path) {
            let Some(file_facts) = facts.and_then(|facts| facts.ts.get(path)) else {
                anyhow::bail!("{RULE_ID}: missing prepared facts for CSP file {file}");
            };
            if let Some(message) = &file_facts.parse_error {
                return Err(csp_input_error(
                    Some(PathBuf::from(&file)),
                    format!(
                        "{message}. Repair the CSP file so {RULE_ID} can read its directives."
                    ),
                ));
            }
            CspPolicySet::parse_from_strings(&file_facts.csp_allowed_origins.csp_strings)
        } else {
            let text = sources.read_source_path(path).map_err(|err| {
                csp_input_error(
                    Some(PathBuf::from(&file)),
                    format!("failed to read CSP file '{file}': {err}. Provide an existing path in options.cspFiles."),
                )
            })?;
            CspPolicySet::parse_from_text(&text)
        };
        if parsed.is_empty() {
            return Err(csp_input_error(
                Some(PathBuf::from(&file)),
                format!("{RULE_ID}: no static CSP directives found in {file}. Declare static directives such as default-src or img-src."),
            ));
        }
        combined.extend(parsed);
    }
    Ok(combined)
}

fn csp_input_error(file: Option<PathBuf>, message: String) -> anyhow::Error {
    CheckInputFailure {
        file,
        property: Some("cspFiles".into()),
        message,
    }
    .into()
}
