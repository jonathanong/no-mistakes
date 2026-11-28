use super::*;
use std::fs::OpenOptions;
use std::io::{Read, Write};

#[path = "../../../../../no-mistakes-invocation/tests/support/blocked_watchdog.rs"]
mod blocked_watchdog;

const HELPER: &str = "invocation::tests::blocked_check::blocked_check_subprocess_helper";
const CONTROL: &str = "NO_MISTAKES_BLOCKED_CHECK_ROOT";

#[test]
fn blocked_check_subprocess_helper() {
    let Some(root) = std::env::var_os(CONTROL).map(PathBuf::from) else {
        return;
    };
    let config = root.join("blocked.yml");
    let cli = Cli::parse_from([
        "no-mistakes",
        "--jobs",
        "1",
        "check",
        "--root",
        root.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
    ]);
    no_mistakes::cli::init_rayon_threads(cli.jobs);
    let worker = std::thread::spawn(|| {
        run_command(cli.command).expect("blocked check must not return before its deadline")
    });
    // Retain this writer without data to keep the real check blocked; see CONTRIBUTING.md.
    let _writer = OpenOptions::new().write(true).open(config).unwrap();
    assert!(!worker.is_finished(), "config loading must remain blocked");
    println!("blocked check ready");
    std::io::stdout().flush().unwrap();
    let mut start = [0];
    std::io::stdin().read_exact(&mut start).unwrap();
    assert_eq!(start, [1]);
    let _execution = ExecutionGuard::acquire_for_cli(no_mistakes::invocation::InvocationOptions {
        timeout: Some(Duration::from_secs(1)),
        ..Default::default()
    })
    .unwrap();
    let code = worker.join().unwrap();
    panic!("watchdog failed to stop blocked check: {code:?}");
}

#[test]
fn check_timeout_stops_a_blocked_repository_check_and_names_the_phase() {
    blocked_watchdog::assert_blocked_watchdog(HELPER, CONTROL, "blocked check ready");
}
