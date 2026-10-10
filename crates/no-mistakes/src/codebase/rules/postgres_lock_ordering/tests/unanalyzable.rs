use super::*;
use crate::codebase::rules::RuleFinding;

fn unanalyzable_findings(options: &str) -> Vec<RuleFinding> {
    let root = fixture("unanalyzable");
    let file = ts_file(&root);
    let mut findings = check_with_files(
        &root,
        &config_with_options(options),
        std::slice::from_ref(&file),
    )
    .unwrap();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&file));
    crate::codebase::rules::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    findings
}

fn lines_and_targets(findings: &[RuleFinding]) -> Vec<(usize, &str)> {
    findings
        .iter()
        .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
        .collect()
}

#[test]
fn opaque_dynamic_sql_fails_closed_by_default() {
    let findings = unanalyzable_findings("{}");
    // The non-locking dynamic read, the safe-directive call and the suppressed call stay quiet.
    assert_eq!(
        lines_and_targets(&findings),
        [(6, UNANALYZABLE_TARGET), (16, LOCK_ORDERING_TARGET)],
        "{findings:#?}"
    );
    let message = &findings[0].message;
    assert!(message.starts_with("src/lock.ts:6: "), "{message}");
    assert!(message.contains("`deadlock-safe`"), "{message}");
    assert!(message.contains("unanalyzableSql: ignore"), "{message}");
}

#[test]
fn ignore_skips_opaque_sql_but_keeps_recovered_lock_checks() {
    let findings = unanalyzable_findings("unanalyzableSql: ignore");
    assert_eq!(
        lines_and_targets(&findings),
        [(16, LOCK_ORDERING_TARGET)],
        "{findings:#?}"
    );
}

#[test]
fn unanalyzable_option_defaults_to_fail_and_rejects_unknown_modes() {
    assert!(
        compile_options(&Options::default())
            .unwrap()
            .fail_unanalyzable
    );
    let ignore = Options {
        unanalyzable_sql: "ignore".to_string(),
        ..Default::default()
    };
    assert!(!compile_options(&ignore).unwrap().fail_unanalyzable);
    let invalid = Options {
        unanalyzable_sql: "sometimes".to_string(),
        ..Default::default()
    };
    let error = compile_options(&invalid).err().unwrap();
    assert!(
        error
            .to_string()
            .contains("postgres-lock-ordering: unanalyzableSql must be `fail` or `ignore`"),
        "{error}"
    );
}
