use super::*;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, ExitStatus};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use crate::saved_fixture;

const HELPER: &str = "cli_invocation::fsmonitor::hung_fsmonitor_subprocess";
const CONTROL: &str = "NO_MISTAKES_HUNG_FSMONITOR_FIXTURE";
// These waits run only until the event arrives. Ten seconds avoids CI flakes
// without slowing a successful run.
const BOUND: Duration = Duration::from_secs(10);

pub(super) fn run() {
    // The parent owns temporary files even if its helper must exit forcibly.
    let fixture = saved_fixture::materialize("invocation", "hung-fsmonitor");
    // Linux subreaper state must never affect the parallel integration harness.
    let mut helper = ManagedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", HELPER, "--nocapture"])
            .env(CONTROL, fixture.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let status = match helper.0.wait_timeout(Duration::from_secs(35)).unwrap() {
        Some(status) => status,
        None => {
            // Let the isolated helper reap its descendants before stopping it.
            unsafe {
                libc::kill(helper.0.id() as i32, libc::SIGTERM);
            }
            let status = helper.0.wait_timeout(BOUND).unwrap();
            panic!("scenario exceeded backstop; cleanup status: {status:?}");
        }
    };
    let mut output = String::new();
    helper
        .0
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    helper
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    assert!(status.success(), "{output}");
}

#[test]
fn hung_fsmonitor_subprocess() {
    let Some(fixture) = std::env::var_os(CONTROL).map(PathBuf::from) else {
        return;
    };
    #[cfg(target_os = "linux")]
    assert_eq!(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1) }, 0);

    let root = fixture.join("repo");
    git(&root, &["init", "-q"]);
    git(&root, &["add", "."]);
    let hook = fixture.join("fsmonitor.sh");
    git(&root, &["config", "core.fsmonitor", hook.to_str().unwrap()]);
    git(&root, &["config", "core.fsmonitorHookVersion", "2"]);
    let identity = fixture.join("hook.pid");
    let (children, _abort) = install_failure_cleanup(identity.clone());
    let mut cleanup = HookCleanup(Some(identity.clone()));
    let home = tempfile::tempdir().unwrap();

    let holder_stderr = fixture.join("holder.stderr");
    let mut holder = check(&root, home.path());
    let mut holder = ManagedChild(
        holder
            .args(["--timeout", "0", "--invocation-timeout-ms", "10000"])
            .env("NO_MISTAKES_FSMONITOR_PID", &identity)
            .stderr(fs::File::create(&holder_stderr).unwrap())
            .spawn()
            .unwrap(),
    );
    children[0].store(holder.0.id() as i32, Ordering::SeqCst);
    let (hook_pid, git_pid) = wait_for_hook(&identity);

    // Only the holder uses the hanging repository. Both invocations share the
    // same isolated user lock, so the contender can finish after its release.
    let clean = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-runner/empty");
    let mut contender = ManagedChild(check(&clean, home.path()).spawn().unwrap());
    children[1].store(contender.0.id() as i32, Ordering::SeqCst);
    let (sender, receiver) = mpsc::channel();
    let pipe = contender.0.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines() {
            if sender.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let expected = format!("waiting for lock held by pid {}", holder.0.id());
    let started = Instant::now();
    loop {
        let line = receiver
            .recv_timeout(BOUND.saturating_sub(started.elapsed()))
            .expect("contender must report contention before the holder expires");
        if line.starts_with(&expected) {
            break;
        }
    }
    assert!(
        holder.0.try_wait().unwrap().is_none(),
        "holder already exited"
    );
    let status = holder.wait(Duration::from_secs(12));
    let error = fs::read_to_string(holder_stderr).unwrap();
    assert_eq!(status.code(), Some(124), "{error}");
    assert!(
        error.contains("timed out") || error.contains("deadline elapsed"),
        "{error}"
    );
    assert!(contender.wait(BOUND).success());
    reader.join().unwrap();
    assert_hook_reaped(hook_pid);
    #[cfg(target_os = "linux")]
    reap_adopted(git_pid);
    #[cfg(not(target_os = "linux"))]
    assert_hook_reaped(git_pid);
    cleanup.0 = None;
}

fn check(root: &Path, home: &Path) -> Command {
    let mut command = Command::new(bin());
    command
        .args(["check", "--root"])
        .arg(root)
        .args(["--lock-timeout", "20"])
        // Cargo's inherited marker otherwise creates a different lock per PID.
        .env_remove("CARGO_BIN_EXE_no-mistakes")
        .env("HOME", home)
        .env("XDG_CACHE_HOME", home)
        .env("XDG_RUNTIME_DIR", home)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
}

fn hook_identity(path: &Path) -> Option<(i32, i32)> {
    let text = fs::read_to_string(path).ok()?;
    let mut fields = text.split_whitespace();
    let pid = fields.next()?.parse().ok()?;
    let parent = fields.next()?.parse().ok()?;
    (pid > 0 && parent > 0).then_some((pid, parent))
}

fn wait_for_hook(path: &Path) -> (i32, i32) {
    let started = Instant::now();
    loop {
        if let Some(identity) = hook_identity(path) {
            return identity;
        }
        assert!(started.elapsed() < BOUND, "fsmonitor hook never started");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn assert_hook_reaped(pid: i32) {
    let started = Instant::now();
    loop {
        #[cfg(target_os = "linux")]
        {
            let mut status = 0;
            let waited = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
            if waited == pid {
                assert!(libc::WIFSIGNALED(status));
                assert_eq!(libc::WTERMSIG(status), libc::SIGKILL);
                return;
            }
            if waited == -1 {
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ECHILD)
                );
                // Git may have reaped it before exiting; otherwise adoption
                // can lag the holder's exit, so keep observing until bounded.
                if unsafe { libc::kill(pid, 0) } == -1 {
                    assert_eq!(
                        std::io::Error::last_os_error().raw_os_error(),
                        Some(libc::ESRCH)
                    );
                    return;
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        if unsafe { libc::kill(pid, 0) } == -1 {
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
            return;
        }
        assert!(started.elapsed() < BOUND, "hook {pid} was not reaped");
        std::thread::sleep(Duration::from_millis(10));
    }
}

struct ManagedChild(Child);

impl ManagedChild {
    fn wait(&mut self, timeout: Duration) -> ExitStatus {
        self.0
            .wait_timeout(timeout)
            .unwrap()
            .expect("child exceeded backstop")
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(target_os = "linux")]
fn reap_adopted(pid: i32) {
    let started = Instant::now();
    loop {
        let waited = unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) };
        if waited == pid
            || (waited == -1
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD))
        {
            return;
        }
        assert!(started.elapsed() < BOUND, "child {pid} was not reaped");
        std::thread::sleep(Duration::from_millis(10));
    }
}

struct HookCleanup(Option<PathBuf>);

fn install_failure_cleanup(identity: PathBuf) -> (Arc<[AtomicI32; 2]>, AbortCleanup) {
    let children = Arc::new([AtomicI32::new(0), AtomicI32::new(0)]);
    let tracked = children.clone();
    let aborting = Arc::new(AtomicBool::new(false));
    let requested = aborting.clone();
    let mut signals = signal_hook::iterator::Signals::new([libc::SIGTERM]).unwrap();
    std::thread::spawn(move || {
        if signals.forever().next().is_some() {
            requested.store(true, Ordering::SeqCst);
            for child in tracked.iter() {
                let pid = child.load(Ordering::SeqCst);
                if pid > 0 {
                    unsafe {
                        libc::kill(pid, libc::SIGKILL);
                    }
                }
            }
            if let Some((pid, parent)) = hook_identity(&identity) {
                unsafe {
                    libc::kill(-parent, libc::SIGKILL);
                    libc::kill(pid, libc::SIGKILL);
                }
            }
            // This subprocess owns no unrelated children. In particular, Linux
            // adoption must finish here, before its subreaper exits on failure.
            let started = Instant::now();
            loop {
                let waited = unsafe { libc::waitpid(-1, std::ptr::null_mut(), libc::WNOHANG) };
                if waited == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD)
                {
                    break;
                }
                if started.elapsed() >= Duration::from_secs(4) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            std::process::exit(1);
        }
    });
    (children, AbortCleanup(aborting))
}

struct AbortCleanup(Arc<AtomicBool>);

impl Drop for AbortCleanup {
    fn drop(&mut self) {
        if self.0.load(Ordering::SeqCst) {
            // Killing the CLIs wakes scenario assertions. Do not let their
            // unwind exit the harness before the cleanup thread reaps children.
            let started = Instant::now();
            while started.elapsed() < BOUND {
                std::thread::sleep(Duration::from_millis(10));
            }
            std::process::exit(1);
        }
    }
}

impl Drop for HookCleanup {
    fn drop(&mut self) {
        if let Some((pid, parent)) = self.0.as_deref().and_then(hook_identity) {
            // Independent failure cleanup must never turn a leaked child into
            // a passing assertion. Git and its hook share this process group.
            unsafe {
                libc::kill(-parent, libc::SIGKILL);
                libc::kill(pid, libc::SIGKILL);
            }
            #[cfg(target_os = "linux")]
            {
                reap_adopted(parent);
                reap_adopted(pid);
            }
        }
    }
}
