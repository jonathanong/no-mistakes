use super::*;

pub(super) fn scan(
    root: &Path,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<ClassifiedFinding> {
    let mut findings: Vec<ClassifiedFinding> = files
        .par_iter()
        .flat_map(
            |path| match path.file_name().and_then(|name| name.to_str()) {
                Some("pnpm-workspace.yaml") => check_workspace(root, path, files, sources),
                Some("package.json") => check_package_json(root, path, sources),
                _ => Vec::new(),
            },
        )
        .collect();
    // A selected included override file stays in scope when the root workspace
    // file itself is filtered out.
    let root_yaml = crate::codebase::ts_resolver::normalize_path(&root.join("pnpm-workspace.yaml"));
    let root_selected = files
        .iter()
        .any(|candidate| crate::codebase::ts_resolver::normalize_path(candidate) == root_yaml);
    if !root_selected {
        if let Ok(source) = sources.read_path(&root_yaml) {
            findings.extend(included::included_override_findings(
                root, &root_yaml, &source, sources, files,
            ));
        }
    }
    findings
}

fn check_workspace(
    root: &Path,
    path: &Path,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<ClassifiedFinding> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(error) => {
            return vec![super::super::manifest_analysis::unavailable(
                root, path, RULE_ID, error,
            )]
        }
    };
    match serde_yaml::from_str::<serde_yaml::Value>(&source) {
        Ok(value) => {
            let mut findings = Vec::new();
            if yaml_has_key(&value, "overrides") {
                findings.push(ClassifiedFinding::Definite(finding(
                    root,
                    path,
                    yaml_top_level_key_line(&source, "overrides"),
                    format!("top-level \"overrides\" is banned. {FIX}"),
                )));
            }
            findings.extend(included::included_override_findings(
                root, path, &source, sources, files,
            ));
            findings
        }
        Err(error) => vec![super::super::manifest_analysis::unavailable(
            root,
            path,
            RULE_ID,
            format!("failed to parse YAML: {error}"),
        )],
    }
}

fn check_package_json(
    root: &Path,
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<ClassifiedFinding> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(error) => {
            return vec![super::super::manifest_analysis::unavailable(
                root, path, RULE_ID, error,
            )]
        }
    };
    let json = match sources.parse_json_path(path) {
        Ok(json) => json,
        Err(error) => {
            return vec![super::super::manifest_analysis::unavailable(
                root, path, RULE_ID, error,
            )]
        }
    };
    let Some(object) = json.as_object() else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    if object.contains_key("overrides") {
        findings.push(ClassifiedFinding::Definite(finding(
            root,
            path,
            json_key_line(&source, "overrides"),
            format!("top-level \"overrides\" is banned. {FIX}"),
        )));
    }
    if object
        .get("pnpm")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|pnpm| pnpm.contains_key("overrides"))
    {
        findings.push(ClassifiedFinding::Definite(finding(
            root,
            path,
            json_nested_key_line(&source, "pnpm", "overrides"),
            format!("\"pnpm.overrides\" is banned. {FIX}"),
        )));
    }
    findings
}

pub(super) fn finding(
    root: &Path,
    path: &Path,
    line: usize,
    message: impl Into<String>,
) -> RuleFinding {
    let file = relative_slash_path(root, path);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.clone(),
        line,
        message: format!("{file}: {}", message.into()),
        import: None,
        target: None,
    }
}
