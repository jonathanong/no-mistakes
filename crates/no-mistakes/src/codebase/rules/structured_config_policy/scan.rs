use super::ancestor_override_subset::{check_ancestor_override_subset, ParsedAncestorCache};
use super::equals_file::check_equals_file;
use super::partial;
use super::paths::CanonicalInventory;
use super::when::policy_applies;
use super::{assert_value, AssertionKind, Options, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct ScanState {
    pub(super) parsed_ancestors: ParsedAncestorCache,
    pub(super) canonical_inventory: Option<CanonicalInventory>,
}

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    inventory: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    scan_with_state(
        root,
        opts,
        files,
        inventory,
        target_roots,
        sources,
        &mut ScanState::default(),
    )
}

pub(super) fn scan_with_state(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    inventory: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    state: &mut ScanState,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for policy in &opts.policies {
        let matching = super::super::matching_files(root, &policy.files, files, target_roots)?;
        for path in matching {
            let rel = relative_slash_path(root, &path);
            let source =
                match super::super::source_access::checked_source(sources, &path, root, RULE_ID) {
                    Ok(source) => source,
                    Err(finding) => {
                        findings.push(*finding);
                        continue;
                    }
                };
            let (value, partial) =
                match crate::codebase::structured_value::parse_structured_value_located(&path, &source) {
                    Ok(value) => (value, false),
                    Err(mut error) if error.missing_merge_location => {
                        let partial = error.partial.take();
                        findings.push(super::merge_location_finding(root, &path, error));
                        let Some(value) = partial else { continue };
                        (*value, true)
                    }
                    Err(error) => {
                        findings.push(RuleFinding {
                            causes: Vec::new(),
                            rule: RULE_ID.to_string(),
                            file: rel.clone(),
                            line: error.line.unwrap_or(1),
                            message: format!("{rel}: {}", error.message),
                            import: None,
                            target: None,
                        });
                        continue;
                    }
                };
            if !policy_applies(&value, &policy.when)
                || (partial && !partial::conditions_known(&value, &policy.when))
            {
                continue;
            }
            for key in &policy.required_keys {
                if partial::key(&value, key, partial) == Some(None) {
                    findings.push(RuleFinding {
                        causes: Vec::new(),
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
                if partial::key(&value, key, partial).flatten().is_some() {
                    findings.push(RuleFinding {
                        causes: Vec::new(),
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
                if partial && assertion.kind == Some(AssertionKind::AncestorOverrideSubset) {
                    findings.extend(super::ancestor_override_subset::validate_partial_extends(
                        &value, &rel, assertion,
                    ));
                    continue;
                }
                if partial && !partial::assertion_known(&value, assertion) {
                    continue;
                }
                match assertion.kind {
                    Some(AssertionKind::EqualsFile) => {
                        findings.extend(check_equals_file(root, &rel, sources, &value, assertion));
                    }
                    Some(AssertionKind::AncestorOverrideSubset) => {
                        let canonical_inventory = state
                            .canonical_inventory
                            .get_or_insert_with(|| CanonicalInventory::new(root, inventory));
                        findings.extend(check_ancestor_override_subset(
                            &path,
                            &rel,
                            sources,
                            &value,
                            assertion,
                            canonical_inventory,
                            &mut state.parsed_ancestors,
                        ));
                    }
                    _ => findings.extend(assert_value(&rel, &value, assertion)?),
                }
            }
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}
