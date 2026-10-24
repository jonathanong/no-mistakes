use super::*;

pub(super) fn check_alias(
    root: &Path,
    tsconfig_str: &str,
    line: usize,
    alias: &str,
    targets: &[String],
    opts: &Options,
    all_paths: &BTreeMap<String, Vec<String>>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();

    // Check alias → target direction
    for mapping in &opts.mappings {
        let alias_prefix = format!("{}/", mapping.prefix);
        if alias.starts_with(&alias_prefix) {
            let suffix = &alias[alias_prefix.len()..];
            let expected = build_path(&[&opts.base_dir, &mapping.root, suffix]);
            for target in targets {
                if !target_matches(&expected, target) {
                    findings.push(make_finding(
                        tsconfig_str,
                        line,
                        &format!("{tsconfig_str}: {alias} must target {expected}, not {target}"),
                    ));
                } else if opts.check_exists && !target_exists(root, &expected, target) {
                    findings.push(make_finding(
                        tsconfig_str,
                        line,
                        &format!("{tsconfig_str}: {alias} targets {target}, which does not exist on disk"),
                    ));
                }
            }
        }
    }

    // Check target → alias direction
    for mapping in &opts.mappings {
        let target_prefix = format!("{}/", build_path(&[&opts.base_dir, &mapping.root]));
        let required_alias_prefix = format!("{}/", mapping.prefix);
        for target in targets {
            if target.starts_with(&target_prefix) {
                // This target belongs to this mapping; the alias must use the right prefix
                if !alias.starts_with(&required_alias_prefix) {
                    // Avoid double-reporting: only flag if not already caught from a different mapping
                    let already_flagged = findings.iter().any(|f: &RuleFinding| {
                        f.message.contains(alias) && f.message.contains(target)
                    });
                    if !already_flagged {
                        // Check the alias isn't valid for any mapping by looking at all known targets
                        let all_targets =
                            all_paths.get(alias).map(Vec::as_slice).unwrap_or(targets);
                        let valid = opts.mappings.iter().any(|m| {
                            let ap = format!("{}/", m.prefix);
                            if alias.starts_with(&ap) {
                                let suffix = &alias[ap.len()..];
                                let expected = build_path(&[&opts.base_dir, &m.root, suffix]);
                                all_targets
                                    .iter()
                                    .any(|target| target_matches(&expected, target))
                            } else {
                                false
                            }
                        });
                        if !valid {
                            findings.push(make_finding(
                                tsconfig_str,
                                line,
                                &format!(
                                    "{tsconfig_str}: {alias} must use prefix {}, not target {target} directly",
                                    mapping.prefix
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    findings
}
