use super::scripts::{scan_script, ScriptHitKind, WorkingDir};

mod later;

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
