use super::*;
use no_mistakes::codebase::ts_source::{FileInventory, SourceStore};
use no_mistakes::diagnostics::InvocationObserver;
use no_mistakes::invocation::InvocationOptions;

#[test]
fn partial_timeout_subprocess_helper() {
    let Ok(mode) = std::env::var("NO_MISTAKES_PARTIAL_TIMEOUT_MODE") else {
        return;
    };
    let mut args = vec!["no-mistakes", "check", "--root", "."];
    if mode.starts_with("verbose") {
        args.push("--verbose-timings");
    }
    if mode.contains("json") {
        args.push("--json");
    }
    use clap::{CommandFactory, FromArgMatches};
    let matches = Cli::command().get_matches_from(args);
    let json = wants_json(&matches);
    let cli = Cli::from_arg_matches(&matches).unwrap();
    let observer = cli.diagnostics.observer();
    assert_eq!(observer.is_some(), mode.starts_with("verbose"));
    assert!(!observer.as_ref().is_some_and(|o| o.progress_enabled()));
    let _scope = observer
        .as_ref()
        .map(|o| InvocationGuard::install(Arc::clone(o)));
    // Read a saved source before starting the watchdog: exact seeded counts
    // must survive process::exit even though no observer guard can unwind.
    seed_saved_source(observer.clone());
    let _stderr_lock = mode.ends_with("locked").then(|| std::io::stderr().lock());
    if mode.ends_with("blocked") && !mode.ends_with("stdoutblocked") {
        use std::io::Write;
        std::io::stderr().write_all(&[b'x'; 4096]).unwrap();
        std::io::stderr().flush().unwrap();
    }
    let _stdout_lock = mode
        .ends_with("stdoutlocked")
        .then(|| std::io::stdout().lock());
    let output = no_mistakes::invocation::CliErrorOutput::new(json);
    let _execution = ExecutionGuard::acquire_for_cli_with_output(
        InvocationOptions {
            timeout: Some(Duration::from_millis(150)),
            invocation_timeout: None,
            ..InvocationOptions::default()
        },
        output.clone(),
    )
    .unwrap();
    if mode.ends_with("stdoutblocked") {
        #[cfg(target_os = "linux")]
        fill_stdout_pipe();
    }
    if mode.contains("boundary") {
        let error =
            std::io::Error::new(std::io::ErrorKind::TimedOut, "boundary deadline elapsed").into();
        report_error(&error, &output);
    }
    std::thread::sleep(Duration::from_secs(10));
    panic!("watchdog failed to enforce its deadline");
}

fn seed_saved_source(observer: Option<Arc<InvocationObserver>>) {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/invocation/not-a-directory");
    let inventory = Arc::new(FileInventory::from_paths(std::slice::from_ref(&path)));
    let store = SourceStore::new_observed(inventory, observer.clone());
    let read = || assert!(!store.read_path(&path).unwrap().is_empty());
    if let Some(observer) = observer {
        observer.trace("fixture.read", TimingKind::Serial, read);
    } else {
        read();
    }
}

#[test]
fn verbose_timeout_without_progress_preserves_partial_counters_and_stays_bounded() {
    for mode in [
        "verbose",
        "off",
        "verbose-locked",
        "off-locked",
        "verbose-blocked",
        "off-blocked",
        "json",
        "verbose-json",
        "json-locked",
        "json-blocked",
        "json-stdoutlocked",
        "json-stdoutblocked",
        "json-boundary",
        "json-boundary-stdoutblocked",
    ] {
        if mode.ends_with("blocked") && !cfg!(target_os = "linux") {
            continue;
        }
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "invocation::tests::timeout::partial_timeout_subprocess_helper",
                "--nocapture",
            ])
            .env("NO_MISTAKES_PARTIAL_TIMEOUT_MODE", mode)
            // This binary unit-test helper calls the normally built library;
            // use its established subprocess lock isolation, as CLI tests do.
            .env(
                "CARGO_BIN_EXE_no-mistakes",
                std::env::current_exe().unwrap(),
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(target_os = "linux")]
        if mode.ends_with("blocked") {
            use std::os::unix::process::CommandExt;
            unsafe {
                command.pre_exec(move || {
                    if nix::libc::fcntl(
                        if mode.ends_with("stdoutblocked") {
                            1
                        } else {
                            2
                        },
                        nix::libc::F_SETPIPE_SZ,
                        4096,
                    ) == -1
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
        let mut child = command.spawn().unwrap();
        let status = child
            .wait_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_or_else(|| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("hard timeout blocked on stderr: {mode}");
            });
        // Drain only after exit; reading a full pipe sooner would hide the bug.
        let output = child.wait_with_output().unwrap();
        assert_eq!(
            status.code(),
            Some(124),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        if mode.contains("json") && !mode.ends_with("stdoutblocked") {
            let report: serde_json::Value = serde_json::from_str(
                stdout
                    .lines()
                    .find(|line| line.starts_with('{'))
                    .expect("JSON timeout report"),
            )
            .unwrap();
            assert_eq!(report["error"]["kind"], "deadline", "{mode}");
            assert_eq!(report["error"]["exitCode"], 124, "{mode}");
            assert!(
                report["error"]["phase"].is_null() || report["error"]["phase"] == "command",
                "{mode}"
            );
            assert_eq!(
                stdout.lines().filter(|line| line.starts_with('{')).count(),
                1,
                "{stdout}"
            );
        } else if !mode.ends_with("stdoutblocked") {
            assert_eq!(stdout.trim(), "running 1 test", "no command stdout: {mode}");
        }
        if mode.ends_with("blocked") {
            continue;
        }
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("command timed out"), "{mode}: {stderr}");
        assert!(!stderr.contains("[progress]"), "{stderr}");
        if mode.starts_with("verbose") {
            assert!(
                stderr.contains("[PARTIAL work] source.reads: 1\n"),
                "{stderr}"
            );
            assert!(
                stderr.contains("[PARTIAL timing] fixture.read:"),
                "{stderr}"
            );
            assert!(stderr.contains("lower bounds"), "{stderr}");
        } else {
            assert!(!stderr.contains("[PARTIAL"), "{stderr}");
        }
    }
}

#[cfg(target_os = "linux")]
fn fill_stdout_pipe() {
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    // Fill the inherited pipe without blocking before the boundary can claim.
    // Restore its original status flags before testing the blocking JSON write.
    unsafe {
        let flags = nix::libc::fcntl(1, nix::libc::F_GETFL);
        assert!(flags >= 0);
        assert_eq!(
            nix::libc::fcntl(1, nix::libc::F_SETFL, flags | nix::libc::O_NONBLOCK),
            0
        );
        let bytes = [b'x'; 4096];
        while nix::libc::write(1, bytes.as_ptr().cast(), bytes.len()) >= 0 {}
        // Atomic 4096-byte writes may fail while a smaller remainder is free.
        while nix::libc::write(1, bytes.as_ptr().cast(), 1) >= 0 {}
        assert_eq!(
            std::io::Error::last_os_error().kind(),
            std::io::ErrorKind::WouldBlock
        );
        assert_eq!(nix::libc::fcntl(1, nix::libc::F_SETFL, flags), 0);
    }
}
