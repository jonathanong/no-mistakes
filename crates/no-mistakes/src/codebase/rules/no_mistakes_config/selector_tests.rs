use super::*;

fn run(config: &str) -> Vec<RuleFinding> {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/forbidden-dependencies-globs/fixture"),
    );
    let path = root.join(config);
    // The config validator is a filesystem rule; run both public rule engines.
    let mut findings = crate::codebase::rules::run_filesystem_rules(&root, Some(&path)).unwrap();
    findings.extend(crate::codebase::rules::run_check(&root, Some(&path), None).unwrap());
    findings
}

#[test]
fn forbidden_selectors_preserve_existing_literal_bracket_files_and_directories() {
    let findings = run("literal-roots.yml");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn forbidden_selector_fallback_still_reports_a_missing_bracket_glob() {
    let findings = run("missing-selector.yml");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, "missing-selector.yml");
    assert_eq!(findings[0].line, 7);
    assert!(findings[0]
        .message
        .contains("options.roots[1]: missing path `missing/[ab]/**/*.mts`"));
}

#[test]
fn forbidden_and_allowed_targets_keep_strict_glob_semantics() {
    let findings = run("strict-target-globs.yml");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings.iter().all(|finding| finding.rule == RULE_ID
        && finding
            .message
            .ends_with("missing path `app/[tenant]/entry.mts`")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains(".forbiddenFiles[0]")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains(".exceptions[0].allowedFiles[0]")));
}
