use super::super::{Inputs, Location, RuleFinding};
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::ts_source::has_disable_file_comment;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn unresolved_binding_message(relative: &str, id: &str, location: &Location) -> String {
    let binding = location
        .playwright_binding
        .as_deref()
        .unwrap_or("test");
    if let Some(reason) = location.unresolved_reason {
        return format!(
            "{relative}:{}: cannot verify whether `{binding}` is Playwright's `test` ({reason}); bind it with a direct `const` chain from Playwright or suppress with `no-mistakes-disable-next-line {id}`",
            location.line
        );
    }
    if let Some((module, export)) = location.resolved_playwright_exports.first() {
        return format!(
            "{relative}:{}: cannot verify whether imported binding `{binding}` (from `{module}` export `{export}`) is Playwright's `test`; if it is a Playwright fixture, add `{{ module: \"{module}\", export: \"{export}\" }}` to `{id}` `options.fixtures`",
            location.line
        );
    }
    let (specifier, export) = location
        .playwright_import
        .as_ref()
        .map_or(("<unknown>", "test"), |import| {
            (import.specifier.as_ref(), import.name.as_ref())
        });
    format!(
        "{relative}:{}: cannot verify whether imported binding `{binding}` (from `{specifier}` export `{export}`) is Playwright's `test`; if it is a Playwright fixture, add its repository-relative module and export to `{id}` `options.fixtures`",
        location.line
    )
}

pub(in crate::codebase::rules::test_reliability) fn unreadable(
    inputs: &Inputs<'_>,
    path: &Path,
    relative: &str,
    id: &str,
    message: String,
) -> Result<Vec<RuleFinding>> {
    let message = format!("{message}. Repair this file so {id} can verify its test settings.");
    if inputs
        .sources
        .read_path(path)
        .is_ok_and(|source| has_disable_file_comment(&source, id))
    {
        return Ok(vec![RuleFinding {
            rule: id.into(),
            file: relative.into(),
            line: 1,
            message,
            import: None,
            target: Some("unanalyzable".into()),
            causes: Vec::new(),
        }]);
    }
    Err(CheckInputFailure {
        file: Some(PathBuf::from(relative)),
        property: None,
        message,
    }
    .into())
}
