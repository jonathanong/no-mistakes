use super::ancestor_override_subset::{check_ancestor_override_subset, ParsedAncestorCache};
use super::equals_file::check_equals_file;
use super::partial;
use super::paths::CanonicalInventory;
use super::when::policy_applies;
use super::{assert_value, AssertionKind, Options, RULE_ID};
use crate::codebase::rules::file_diagnostics::ClassifiedInputFindings;
use no_mistakes_rule_kernel::finding::RuleFinding;
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
) -> Result<ClassifiedInputFindings> {
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
) -> Result<ClassifiedInputFindings> {
    let mut result = ClassifiedInputFindings::default();
    for policy in &opts.policies {
        let matching = super::super::matching_files(root, &policy.files, files, target_roots)?;
        for path in matching {
            let rel = relative_slash_path(root, &path);
            let source =
                match super::super::source_access::checked_source(sources, &path, root, RULE_ID) {
                    Ok(source) => source,
                    Err(finding) => {
                        result.findings.push(*finding);
                        continue;
                    }
                };
            let parsed =
                crate::codebase::structured_value::parse_structured_value_located(&path, &source);
            let (value, partial) = match &parsed {
                Ok(value) => (value, None),
                Err(error) if error.missing_merge_location => {
                    result
                        .uncertainty
                        .push(super::merge_location_finding(root, &path, error));
                    let Some(partial) = error.partial.as_deref() else {
                        continue;
                    };
                    (partial.value(), Some(partial))
                }
                Err(error) => {
                    result.findings.push(RuleFinding {
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
            if !policy_applies(value, &policy.when)
                || partial.is_some_and(|partial| !partial::conditions_known(partial, &policy.when))
            {
                continue;
            }
            for key in &policy.required_keys {
                if partial::key(value, key, partial) == Some(None) {
                    result.findings.push(RuleFinding {
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
                if partial::key(value, key, partial).flatten().is_some() {
                    result.findings.push(RuleFinding {
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
                if let Some(partial) = partial {
                    if !partial::assertion_known(partial, assertion) {
                        if assertion.kind == Some(AssertionKind::AncestorOverrideSubset) {
                            result.findings.extend(
                                super::ancestor_override_subset::validate_partial_extends(
                                    partial, &rel, assertion,
                                ),
                            );
                        }
                        continue;
                    }
                }
                match assertion.kind {
                    Some(AssertionKind::EqualsFile) => result.findings.extend(check_equals_file(
                        root,
                        &rel,
                        sources,
                        value,
                        assertion,
                        &mut result.uncertainty,
                    )),
                    Some(AssertionKind::AncestorOverrideSubset) => {
                        let canonical_inventory = state
                            .canonical_inventory
                            .get_or_insert_with(|| CanonicalInventory::new(root, inventory));
                        let ancestor = check_ancestor_override_subset(
                            &path,
                            &rel,
                            sources,
                            value,
                            assertion,
                            canonical_inventory,
                            &mut state.parsed_ancestors,
                        );
                        result.findings.extend(ancestor.findings);
                        result.uncertainty.extend(ancestor.uncertainty);
                    }
                    _ => result
                        .findings
                        .extend(assert_value(&rel, value, assertion)?),
                }
            }
        }
    }
    super::super::sort_findings::sort_by_file_and_message(&mut result.findings);
    Ok(result)
}
