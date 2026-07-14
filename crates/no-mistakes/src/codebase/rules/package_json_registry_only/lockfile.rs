use super::*;

const BLOCKED_RESOLUTION_KEYS: &[&str] = &["tarball", "repo", "commit", "directory"];

pub(super) fn check(
    root: &Path,
    lockfile_root: &Path,
    opts: &Options,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let Some(lockfile_path) = &opts.lockfile else {
        return Vec::new();
    };
    let lockfile_abs = lockfile_root.join(lockfile_path);
    let Some(content) = super::super::read_source(sources, &lockfile_abs) else {
        return Vec::new();
    };
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return Vec::new();
    };
    let file = relative_slash_path(root, &lockfile_abs);
    let Some(packages) = yaml.get("packages").and_then(|p| p.as_mapping()) else {
        return Vec::new();
    };
    let mut pairs: Vec<(&serde_yaml::Value, &serde_yaml::Value)> = packages.iter().collect();
    pairs.sort_by(|(a, _), (b, _)| a.as_str().unwrap_or("").cmp(b.as_str().unwrap_or("")));
    let mut findings = Vec::new();
    for (key, pkg_val) in pairs {
        let pkg_name = key.as_str().unwrap_or("");
        let Some(resolution) = pkg_val.get("resolution") else {
            continue;
        };
        for &blocked_key in BLOCKED_RESOLUTION_KEYS {
            if resolution.get(blocked_key).is_some() {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.clone(),
                    line: 1,
                    message: format!(
                        "{file}: package \"{pkg_name}\" has a non-registry \
                        resolution ({blocked_key}) \u{2014} only npm registry packages are permitted"
                    ),
                    import: None,
                    target: None,
                });
                break;
            }
        }
    }
    findings
}
