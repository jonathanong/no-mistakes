use super::REACT_HOOKS;
use crate::codebase::ts_source::facts::react_analysis;

#[test]
fn installs_hooks_before_constructing_a_fact_context() {
    const CHILD_ENV: &str = "NM_REACT_HOOK_STARTUP_CHILD";
    const TEST: &str =
        "react_analysis_registration::tests::installs_hooks_before_constructing_a_fact_context";
    if std::env::var_os(CHILD_ENV).is_some() {
        assert!(std::ptr::eq(react_analysis::hooks(), &REACT_HOOKS));
        return;
    }

    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST])
        .env(CHILD_ENV, "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&child.stdout);
    assert!(
        child.status.success() && stdout.contains("1 passed"),
        "{stdout}"
    );
}
