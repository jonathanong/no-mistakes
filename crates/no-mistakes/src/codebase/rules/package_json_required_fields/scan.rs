use super::{pkg_rel, Options, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    manifests: &[PathBuf],
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for path in manifests {
        if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            continue;
        }
        let Ok(json) = sources.parse_json_path(path) else {
            continue;
        };
        let rel = pkg_rel(root, path);
        findings.extend(check_package(rel, path, all_files, opts, &json));
    }
    findings
}

fn check_package(
    rel: String,
    path: &Path,
    files: &[PathBuf],
    opts: &Options,
    json: &Value,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let name = json.get("name").and_then(Value::as_str).unwrap_or("");
    if opts.require_scoped_name
        && !opts.unscoped_name_exceptions.iter().any(|n| n == name)
        && !name.starts_with('@')
    {
        findings.push(finding(
            &rel,
            format!("{rel}: package name \"{name}\" must be scoped (start with @)."),
            "name",
        ));
    }
    if let Some(expected) = opts.private {
        if json.get("private").and_then(Value::as_bool) != Some(expected) {
            findings.push(finding(
                &rel,
                format!("{rel}: must declare \"private\": {expected}."),
                "private",
            ));
        }
    }
    if let Some(expected) = &opts.type_value {
        if json.get("type").and_then(Value::as_str) != Some(expected.as_str()) {
            findings.push(finding(
                &rel,
                format!("{rel}: must declare \"type\": \"{expected}\"."),
                "type",
            ));
        }
    }
    if let Some(expected) = &opts.license {
        if json.get("license").and_then(Value::as_str) != Some(expected.as_str()) {
            findings.push(finding(
                &rel,
                format!("{rel}: must declare \"license\": \"{expected}\"."),
                "license",
            ));
        }
    }
    if let Some(main_file) = &opts.main_when_file_exists {
        if sibling_exists(files, path, main_file)
            && json.get("main").and_then(Value::as_str) != Some(main_file.as_str())
        {
            findings.push(finding(
                &rel,
                format!(
                    "{rel}: must declare \"main\": \"{main_file}\" because {main_file} exists in the package directory."
                ),
                "main",
            ));
        }
    }
    findings
}

fn sibling_exists(files: &[PathBuf], package_json: &Path, name: &str) -> bool {
    package_json
        .parent()
        .is_some_and(|dir| files.iter().any(|path| path == &dir.join(name)))
}

fn finding(rel: &str, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
