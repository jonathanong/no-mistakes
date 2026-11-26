use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

struct FakeObserver {
    progress: bool,
    timeout_messages: Mutex<Vec<String>>,
    signaled: AtomicBool,
}

impl FakeObserver {
    fn new(progress: bool) -> Arc<Self> {
        Arc::new(Self {
            progress,
            timeout_messages: Mutex::new(Vec::new()),
            signaled: AtomicBool::new(false),
        })
    }
}

impl DiagnosticsObserver for FakeObserver {
    fn progress_enabled(&self) -> bool {
        self.progress
    }

    fn render_timeout_stderr(self: Arc<Self>, message: String) {
        self.timeout_messages.lock().unwrap().push(message);
    }

    fn render_progress_signal_stderr(&self) {
        self.signaled.store(true, Ordering::SeqCst);
    }
}

struct FakeHooks {
    observer: Option<Arc<FakeObserver>>,
    printed: Mutex<Vec<String>>,
    timeouts: Mutex<Vec<(bool, String)>>,
}

impl DiagnosticsHooks for FakeHooks {
    fn current_observer(&self) -> Option<Arc<dyn DiagnosticsObserver>> {
        self.observer
            .as_ref()
            .map(|observer| Arc::clone(observer) as Arc<dyn DiagnosticsObserver>)
    }

    fn print_stderr(&self, message: Arguments<'_>) {
        self.printed.lock().unwrap().push(message.to_string());
    }

    fn render_timeout_stderr(
        &self,
        observer: Option<Arc<dyn DiagnosticsObserver>>,
        message: String,
    ) {
        self.timeouts
            .lock()
            .unwrap()
            .push((observer.is_some(), message));
    }
}

#[test]
fn unregistered_hooks_use_plain_stderr_and_observer_fallback() {
    assert!(current_observer_from(None).is_none());
    print_stderr_with(None, format_args!("plain message"));
    {
        // A locked stderr must not deadlock the unregistered timeout fallback.
        let _lock = std::io::stderr().lock();
        render_timeout_stderr_with(None, None, "plain timeout".to_string());
    }

    let observer = FakeObserver::new(true);
    assert!(observer.progress_enabled());
    observer.render_progress_signal_stderr();
    assert!(observer.signaled.load(Ordering::SeqCst));
    render_timeout_stderr_with(
        None,
        Some(Arc::clone(&observer) as Arc<dyn DiagnosticsObserver>),
        "observer timeout".to_string(),
    );
    assert_eq!(
        *observer.timeout_messages.lock().unwrap(),
        vec!["observer timeout"]
    );
}

#[test]
fn registered_hooks_receive_observer_stderr_and_timeout_calls() {
    let observer = FakeObserver::new(true);
    let hooks = FakeHooks {
        observer: Some(Arc::clone(&observer)),
        printed: Mutex::new(Vec::new()),
        timeouts: Mutex::new(Vec::new()),
    };

    let current = current_observer_from(Some(&hooks)).expect("observer present");
    assert!(current.progress_enabled());
    print_stderr_with(Some(&hooks), format_args!("waiting {}", 1));
    render_timeout_stderr_with(Some(&hooks), Some(current), "hook timeout".to_string());

    assert_eq!(*hooks.printed.lock().unwrap(), vec!["waiting 1"]);
    assert_eq!(
        *hooks.timeouts.lock().unwrap(),
        vec![(true, "hook timeout".to_string())]
    );
}
