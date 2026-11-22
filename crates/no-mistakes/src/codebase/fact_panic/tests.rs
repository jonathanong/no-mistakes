use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

macro_rules! inject_fact_panic {
    ($path:expr) => {
        crate::codebase::fact_panic::tests::inject($path)
    };
}

static INJECTED: LazyLock<Mutex<BTreeMap<PathBuf, Option<usize>>>> = LazyLock::new(Mutex::default);

pub(super) fn inject(path: &Path) {
    let injected = match INJECTED.lock().unwrap().get_mut(path) {
        Some(Some(attempt)) if *attempt > 0 => {
            *attempt -= 1;
            *attempt == 0
        }
        Some(None) => true,
        _ => false,
    };
    if injected {
        panic!("injected fact extraction failure");
    }
}

pub(crate) fn with_injected_panic<T>(path: &Path, run: impl FnOnce() -> T) -> T {
    with_injected_attempt(path, None, run)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
pub(crate) fn with_injected_panic_on_attempt<T>(
    path: &Path,
    attempt: usize,
    run: impl FnOnce() -> T,
) -> T {
    with_injected_attempt(path, Some(attempt), run)
}

fn with_injected_attempt<T>(path: &Path, attempt: Option<usize>, run: impl FnOnce() -> T) -> T {
    struct Reset(PathBuf);
    impl Drop for Reset {
        fn drop(&mut self) {
            INJECTED.lock().unwrap().remove(&self.0);
        }
    }
    INJECTED.lock().unwrap().insert(path.to_path_buf(), attempt);
    let _reset = Reset(path.to_path_buf());
    run()
}

#[test]
fn panic_payloads_preserve_strings_and_identify_unknown_payloads() {
    let session = super::super::analysis_session::AnalysisSession::disabled();
    let path = Path::new("file.ts");
    for (payload, expected) in [
        (
            Box::new("borrowed") as Box<dyn std::any::Any + Send>,
            "borrowed",
        ),
        (Box::new(String::from("owned")), "owned"),
        (Box::new(42_u8), "non-string panic payload"),
    ] {
        let error =
            super::isolate(&session, path, || std::panic::resume_unwind(payload)).unwrap_err();
        assert!(error.contains(expected));
        assert!(error.contains("file.ts"));
    }
    assert_eq!(session.check_failures().len(), 3);
    assert_eq!(super::isolate(&session, path, || 42).unwrap(), 42);
}

mod collection;
