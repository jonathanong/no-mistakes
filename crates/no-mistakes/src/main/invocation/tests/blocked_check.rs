use super::*;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::Child;
use std::sync::mpsc;
use std::time::Instant;

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
    worker.join().unwrap();
    panic!("watchdog failed to stop blocked check");
}

#[test]
fn check_timeout_stops_a_blocked_repository_check_and_names_the_phase() {
    let root = tempfile::tempdir().unwrap();
    assert!(Command::new("mkfifo")
        .arg(root.path().join("blocked.yml"))
        .status()
        .unwrap()
        .success());
    let mut child = ManagedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", HELPER, "--nocapture"])
            .env(CONTROL, root.path())
            .env(
                "CARGO_BIN_EXE_no-mistakes",
                std::env::current_exe().unwrap(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let (sender, receiver) = mpsc::channel();
    let stdout = child.0.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if line.unwrap() == "blocked check ready" {
                sender.send(()).unwrap();
                break;
            }
        }
    });
    receiver
        .recv_timeout(Duration::from_secs(15))
        .expect("check must reach blocked config loading before its deadline is armed");
    reader.join().unwrap();
    // ChildExt::wait_timeout closes Child::stdin; retain the start gate separately.
    let mut start_gate = child.0.stdin.take().unwrap();
    if let Some(status) = child.0.wait_timeout(Duration::from_millis(1100)).unwrap() {
        let mut stderr = String::new();
        child
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        panic!("blocked check exited before the start gate opened: {status}: {stderr}");
    }
    let started = Instant::now();
    start_gate.write_all(&[1]).unwrap();
    let status = child
        .0
        .wait_timeout(Duration::from_secs(8))
        .unwrap()
        .expect("check kept running past the timeout backstop");
    let elapsed = started.elapsed();
    let mut stderr = String::new();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert_eq!(status.code(), Some(124), "{stderr}");
    assert!(
        stderr.contains("command timed out after 1 seconds during check"),
        "{stderr}"
    );
    assert!(elapsed >= Duration::from_millis(900), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(5), "{elapsed:?} {stderr}");
    assert!(child.0.try_wait().unwrap().is_some());
}

struct ManagedChild(Child);

impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
