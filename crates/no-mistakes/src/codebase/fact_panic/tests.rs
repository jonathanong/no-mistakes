use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

macro_rules! inject_fact_panic {
    ($path:expr) => {
        crate::codebase::fact_panic::tests::inject($path)
    };
}

static INJECTED: LazyLock<Mutex<BTreeSet<PathBuf>>> = LazyLock::new(Mutex::default);

pub(super) fn inject(path: &Path) {
    let injected = INJECTED.lock().unwrap().contains(path);
    if injected {
        panic!("injected fact extraction failure");
    }
}

pub(crate) fn with_injected_panic<T>(path: &Path, run: impl FnOnce() -> T) -> T {
    struct Reset(PathBuf);
    impl Drop for Reset {
        fn drop(&mut self) {
            INJECTED.lock().unwrap().remove(&self.0);
        }
    }
    INJECTED.lock().unwrap().insert(path.to_path_buf());
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
