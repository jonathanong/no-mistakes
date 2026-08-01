//! `RuleFinding` construction for `production-dependency-declarations`.

use super::RULE_ID;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn dev_only(
    root: &Path,
    file: &Path,
    line: u32,
    owning_package: &str,
    imported_package: &str,
    import_specifier: &str,
) -> RuleFinding {
    let rel = relative_slash_path(root, file);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.clone(),
        line: line as usize,
        message: format!(
            "{rel}: imports '{import_specifier}', which {owning_package} declares only under a \
             non-production dependency field (e.g. devDependencies); the production deploy \
             prunes that field, so move it to dependencies (or \
             optionalDependencies/peerDependencies)"
        ),
        import: Some(import_specifier.to_string()),
        target: Some(imported_package.to_string()),
    }
}

pub(super) fn undeclared(
    root: &Path,
    file: &Path,
    line: u32,
    owning_package: &str,
    imported_package: &str,
    import_specifier: &str,
) -> RuleFinding {
    let rel = relative_slash_path(root, file);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.clone(),
        line: line as usize,
        message: format!(
            "{rel}: imports '{import_specifier}', which {owning_package} does not declare as a \
             dependency; add it to dependencies (or optionalDependencies/peerDependencies)"
        ),
        import: Some(import_specifier.to_string()),
        target: Some(imported_package.to_string()),
    }
}

pub(super) fn config(message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: ".no-mistakes.yml".to_string(),
        line: 1,
        message: format!("{RULE_ID}: {message}"),
        import: None,
        target: None,
    }
}
