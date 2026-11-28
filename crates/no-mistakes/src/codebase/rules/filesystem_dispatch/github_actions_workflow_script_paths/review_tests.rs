use super::scripts::{scan_script, ScriptHitKind, WorkingDir};

fn dirs() -> Vec<String> {
    vec!["scripts".to_string()]
}

fn invocation(path: &str) -> impl Fn(&super::scripts::ScriptHit) -> bool + '_ {
    move |hit| {
        matches!(
            &hit.kind,
            ScriptHitKind::Invocation { path: got, .. } if got == path
        )
    }
}

#[test]
fn dash_c_and_filename_globs_do_not_pass_silently() {
    let dash = scan_script(
        "dash -c 'bash scripts/check.sh'\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(dash.iter().any(invocation("scripts/check.sh")), "{dash:?}");
    let glob = scan_script(
        "bash scripts/*.sh\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        glob.iter()
            .any(|hit| matches!(hit.kind, ScriptHitKind::Unanalyzable { .. })),
        "{glob:?}"
    );
    assert!(!glob.iter().any(invocation("scripts/*.sh")), "{glob:?}");
}

#[test]
fn conditional_cd_does_not_rewrite_later_scripts() {
    let hits = scan_script(
        "if false; then\n  cd scripts\nfi\nbash ./scripts/check.sh\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        hits.iter()
            .any(|hit| matches!(hit.kind, ScriptHitKind::Unanalyzable { .. })),
        "{hits:?}"
    );
    assert!(
        !hits.iter().any(invocation("scripts/scripts/check.sh")),
        "{hits:?}"
    );
    let straight = scan_script(
        "cd scripts && bash ./check.sh\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        straight.iter().any(invocation("scripts/check.sh")),
        "{straight:?}"
    );
}

#[test]
fn python_shell_is_unanalyzable() {
    let hits = super::scripts::scan_run(
        "runpy.run_path('scripts/check.py')\n",
        Some("python"),
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        matches!(
            hits.first().map(|hit| &hit.kind),
            Some(ScriptHitKind::Unanalyzable { .. })
        ),
        "{hits:?}"
    );
    let bash = super::scripts::scan_run(
        "bash scripts/check.sh\n",
        Some("bash"),
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(bash.iter().any(invocation("scripts/check.sh")), "{bash:?}");
}

#[test]
fn clustered_shell_c_scans_the_next_word() {
    let hits = scan_script(
        "sh -ce 'bash scripts/nested.sh'\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(hits.iter().any(invocation("scripts/nested.sh")), "{hits:?}");
}

#[test]
fn same_line_cd_applies_only_before_a_substitution() {
    let later = scan_script(
        "cd scripts; echo \"$(./check.sh)\"\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        later.iter().any(invocation("scripts/check.sh")),
        "{later:?}"
    );
    let earlier = scan_script(
        "echo \"$(./check.sh)\"; cd scripts\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(earlier.is_empty(), "{earlier:?}");
}

#[test]
fn cd_options_do_not_become_directories() {
    let physical = scan_script(
        "cd -P scripts\n./check.sh\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        physical.iter().any(invocation("scripts/check.sh")),
        "{physical:?}"
    );
    let unknown = scan_script(
        "cd -x scripts\n./check.sh\n",
        WorkingDir::Known(String::new()),
        &dirs(),
    );
    assert!(
        unknown
            .iter()
            .any(|hit| matches!(hit.kind, ScriptHitKind::Unanalyzable { .. })),
        "{unknown:?}"
    );
    assert!(
        unknown
            .iter()
            .all(|hit| !invocation("scripts/check.sh")(hit)),
        "{unknown:?}"
    );
}

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
            rule: super::RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("scriptDirs: [scripts]\nunanalyzable: report\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let findings = super::check(&root, &config).unwrap();
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
