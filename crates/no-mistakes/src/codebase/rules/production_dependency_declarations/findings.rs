//! `RuleFinding` construction for `production-dependency-declarations`.

use super::RULE_ID;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

pub(super) fn dev_only(
    root: &Path,
    file: &Path,
    line: u32,
    package_name: &str,
    import_name: &str,
) -> RuleFinding {
    let rel = relative_slash_path(root, file);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.clone(),
        line: line as usize,
        message: format!(
            "{rel}: imports '{import_name}', which {package_name} declares only under \
             devDependencies; the production deploy prunes devDependencies, so move it to \
             dependencies (or optionalDependencies/peerDependencies)"
        ),
        import: Some(import_name.to_string()),
        target: Some(package_name.to_string()),
    }
}

pub(super) fn undeclared(
    root: &Path,
    file: &Path,
    line: u32,
    package_name: &str,
    import_name: &str,
) -> RuleFinding {
    let rel = relative_slash_path(root, file);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.clone(),
        line: line as usize,
        message: format!(
            "{rel}: imports '{import_name}', which {package_name} does not declare as a \
             dependency; add it to dependencies (or optionalDependencies/peerDependencies)"
        ),
        import: Some(import_name.to_string()),
        target: Some(package_name.to_string()),
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
