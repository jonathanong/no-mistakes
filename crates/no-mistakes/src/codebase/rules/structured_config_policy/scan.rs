use super::{assert_value, value_at_key, Options, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for policy in &opts.policies {
        let matching = super::super::matching_files(root, &policy.files, files, target_roots)?;
        for path in matching {
            let rel = relative_slash_path(root, &path);
            let Some(source) = super::super::read_source(sources, &path) else {
                continue;
            };
            let value =
                match crate::codebase::structured_value::parse_structured_value(&path, &source) {
                    Ok(value) => value,
                    Err(error) => {
                        findings.push(RuleFinding {
                            rule: RULE_ID.to_string(),
                            file: rel.clone(),
                            line: 1,
                            message: format!("{rel}: {error}"),
                            import: None,
                            target: None,
                        });
                        continue;
                    }
                };
            for key in &policy.required_keys {
                if value_at_key(&value, key).is_none() {
                    findings.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!("{rel}: required config key `{key}` is missing"),
                        import: None,
                        target: Some(key.clone()),
                    });
                }
            }
            for key in &policy.banned_keys {
                if value_at_key(&value, key).is_some() {
                    findings.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!("{rel}: banned config key `{key}` is present"),
                        import: None,
                        target: Some(key.clone()),
                    });
                }
            }
            for assertion in &policy.value_assertions {
                findings.extend(assert_value(&rel, &value, assertion)?);
            }
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}
