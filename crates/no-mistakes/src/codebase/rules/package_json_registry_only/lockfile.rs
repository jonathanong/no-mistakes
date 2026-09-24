use super::*;
use std::collections::BTreeSet;

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
    let file = relative_slash_path(root, &lockfile_abs);
    let docs = match crate::codebase::lockfile::pnpm::load_documents(&content) {
        Ok(docs) => docs,
        Err(_) => return vec![parse_finding(&file, "could not be parsed")],
    };
    if crate::codebase::lockfile::pnpm::validate_env_prefix(&docs).is_err() {
        return vec![parse_finding(
            &file,
            "has a non-env document before the project lockfile",
        )];
    }
    let mut pairs: Vec<(&serde_yaml::Value, &serde_yaml::Value)> = Vec::new();
    for packages in crate::codebase::lockfile::pnpm::package_maps(&docs) {
        pairs.extend(packages.iter());
    }
    if pairs.is_empty() {
        return Vec::new();
    }
    pairs.sort_by(|(a, _), (b, _)| a.as_str().unwrap_or("").cmp(b.as_str().unwrap_or("")));
    let mut findings = Vec::new();
    // The same key can appear in the env document and the project document.
    // One finding per key and blocked resolution; distinct resolutions stay.
    let mut seen = BTreeSet::new();
    for (key, pkg_val) in pairs {
        let pkg_name = key.as_str().unwrap_or("");
        let Some(resolution) = pkg_val.get("resolution") else {
            continue;
        };
        for &blocked_key in BLOCKED_RESOLUTION_KEYS {
            if resolution.get(blocked_key).is_some() {
                if !seen.insert((pkg_name.to_string(), blocked_key.to_string())) {
                    break;
                }
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

fn parse_finding(file: &str, detail: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: format!("{file}: pnpm lockfile {detail}"),
        import: None,
        target: None,
    }
}
