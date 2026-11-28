use super::super::scripts::{scan_script, ScriptHitKind, WorkingDir};
use super::{dirs, invocation};

#[test]
fn absolute_interpreter_keeps_the_script_argument() {
    let hits = scan_script(
        "/usr/bin/python3 scripts/check.py\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(hits.iter().any(invocation("scripts/check.py")), "{hits:?}");
    let inline = scan_script(
        "/usr/bin/python3 -c 'print(1)'\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(inline.is_empty(), "{inline:?}");
}

#[test]
fn package_runner_directory_flag_is_unanalyzable() {
    let hits = scan_script(
        "pnpm --dir scripts exec node check.mjs\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        hits.iter().any(|hit| {
            matches!(&hit.kind, ScriptHitKind::Unanalyzable { token, .. } if token == "--dir")
        }),
        "{hits:?}"
    );
    assert!(
        hits.iter().all(|hit| !invocation("check.mjs")(hit)),
        "{hits:?}"
    );
}

#[test]
fn mid_segment_expansion_can_still_name_a_script_dir() {
    let hits = scan_script(
        "node \"scr${PART}ipts/check.sh\"\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        hits.iter()
            .any(|hit| matches!(hit.kind, ScriptHitKind::Unanalyzable { .. })),
        "{hits:?}"
    );
    let outside = scan_script(
        "node \"other/${FILE}.sh\"\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(outside.is_empty(), "{outside:?}");
}

#[test]
fn shell_nesting_past_four_levels_is_unanalyzable() {
    let shallow = scan_script(
        "echo \"$(echo \"$(echo \"$(./scripts/check.sh)\" )\" )\"\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        shallow.iter().any(invocation("scripts/check.sh")),
        "{shallow:?}"
    );
    let deep = scan_script(
        "echo \"$(echo \"$(echo \"$(echo \"$(echo \"$(./scripts/check.sh)\" )\" )\" )\" )\"\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        deep.iter().any(|hit| {
            matches!(&hit.kind, ScriptHitKind::Unanalyzable { detail, .. } if detail.contains("4 levels"))
        }),
        "{deep:?}"
    );
    assert!(
        deep.iter().all(|hit| !invocation("scripts/check.sh")(hit)),
        "{deep:?}"
    );
}

#[test]
fn composite_depth_limit_stays_unanalyzable_without_blocking_a_shallower_path() {
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };
    use std::path::PathBuf;

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-workflow-script-paths/depth-limit"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: super::super::RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("scriptDirs: [scripts]\nunanalyzable: report\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let findings = super::super::check(&root, &config).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.message.contains("more than 4 levels")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.message.contains("scripts/leaf.sh")),
        "{findings:?}"
    );
}
