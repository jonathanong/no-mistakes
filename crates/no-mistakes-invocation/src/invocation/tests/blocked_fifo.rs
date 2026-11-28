use super::*;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
#[path = "../../../tests/support/blocked_watchdog.rs"]
mod blocked_watchdog;

const HELPER: &str = "invocation::tests::blocked_fifo::blocked_fifo_subprocess_helper";
const CONTROL: &str = "NO_MISTAKES_BLOCKED_FIFO_ROOT";

#[test]
fn blocked_fifo_subprocess_helper() {
    let Some(root) = std::env::var_os(CONTROL).map(PathBuf::from) else {
        return;
    };
    let fifo = root.join("blocked.yml");
    let reader_path = fifo.clone();
    let worker = std::thread::spawn(move || {
        set_timeout_phase("check");
        std::fs::read_to_string(reader_path)
    });
    // Retain a writer without data so the worker blocks inside the FIFO read.
    let _writer = OpenOptions::new().write(true).open(&fifo).unwrap();
    assert!(!worker.is_finished(), "FIFO read must remain blocked");
    println!("blocked fifo ready");
    std::io::stdout().flush().unwrap();
    let mut start = [0];
    std::io::stdin().read_exact(&mut start).unwrap();
    assert_eq!(start, [1]);
    let _execution = InvocationGuard::acquire_for_cli(InvocationOptions {
        timeout: Some(Duration::from_secs(1)),
        ..Default::default()
    })
    .unwrap();
    let outcome = worker.join().unwrap();
    panic!("watchdog failed to stop blocked FIFO read: {outcome:?}");
}

#[test]
fn watchdog_stops_blocked_fifo_read_and_names_check_phase() {
    blocked_watchdog::assert_blocked_watchdog(HELPER, CONTROL, "blocked fifo ready");
}
