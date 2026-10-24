use super::*;

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::super::path_filter::RulePathFilter,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<ClassifiedFinding> {
    let mut findings: Vec<ClassifiedFinding> = files
        .par_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "package.json")
                && !p.components().any(|c| c.as_os_str() == "node_modules")
                && (opts.scopes.is_empty()
                    || opts.scopes.iter().any(|s| {
                        p.starts_with(if s.is_absolute() {
                            s.clone()
                        } else {
                            root.join(s)
                        })
                    }))
        })
        .flat_map(|path| check_package_json_with_sources(path, root, sources))
        .collect();
    findings.extend(
        target_roots
            .par_iter()
            .flat_map(|lockfile_root| {
                let Some(lockfile_path) = &opts.lockfile else {
                    return Vec::new();
                };
                if !rule_filter.is_match(&lockfile_root.join(lockfile_path)) {
                    return Vec::new();
                }
                lockfile::check(root, lockfile_root, opts, sources)
            })
            .collect::<Vec<_>>(),
    );
    findings.sort_by(|a, b| {
        a.finding()
            .file
            .cmp(&b.finding().file)
            .then(a.finding().message.cmp(&b.finding().message))
    });
    findings
}

pub(super) fn check_package_json_with_sources(
    path: &Path,
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<ClassifiedFinding> {
    let json = match sources.parse_json_path(path) {
        Ok(json) => json,
        Err(error) => {
            return vec![super::super::manifest_analysis::unavailable(
                root, path, RULE_ID, error,
            )]
        }
    };
    check_package_json_value(path, root, &json)
}

fn check_package_json_value(
    path: &Path,
    root: &Path,
    json: &serde_json::Value,
) -> Vec<ClassifiedFinding> {
    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();
    for field in DEP_FIELDS {
        let Some(deps) = json.get(field).and_then(|v| v.as_object()) else {
            continue;
        };
        let mut sorted: Vec<_> = deps.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (dep, val) in sorted {
            let spec = val.as_str().unwrap_or("");
            if is_blocked_specifier(spec) {
                findings.push(ClassifiedFinding::Definite(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.clone(),
                    line: 1,
                    message: format!(
                        "{file}: \"{dep}\": \"{spec}\" is not allowed \
                        (only npm registry / workspace: / catalog: / npm: aliases permitted)"
                    ),
                    import: None,
                    target: None,
                }));
            }
        }
    }
    findings
}
