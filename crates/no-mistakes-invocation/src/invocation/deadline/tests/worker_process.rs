use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub(super) fn run_isolated(module: &str, test: &str, guard: &str, body: impl FnOnce()) {
    if std::env::var_os(guard).is_some() {
        body();
        eprintln!("{guard}: completed");
        return;
    }

    let test_name = exact_test_name(module, test);
    let mut child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", &test_name, "--nocapture"])
        .env(guard, "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("isolated test process");
    let status = child
        .wait_timeout(Duration::from_secs(10))
        .expect("wait for isolated test process")
        .unwrap_or_else(|| {
            let _ = child.kill();
            let _ = child.wait();
            panic!("isolated test process exceeded its 10 second bound: {test_name}");
        });
    let output = child.wait_with_output().expect("collect test output");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        status.success(),
        "isolated test process failed: {test_name}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    let marker = format!("{guard}: completed");
    assert!(
        stderr.contains(marker.as_str()),
        "isolated test process did not execute the guarded test: {test_name}\nstderr:\n{stderr}"
    );
}

fn exact_test_name(module: &str, test: &str) -> String {
    let crate_prefix = format!("{}::", env!("CARGO_CRATE_NAME"));
    let module = module.strip_prefix(&crate_prefix).unwrap_or(module);
    format!("{module}::{test}")
}
