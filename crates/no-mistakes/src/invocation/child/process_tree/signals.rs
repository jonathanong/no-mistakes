use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, OnceLock, Weak};

pub(crate) struct SignalRegistry {
    groups: Mutex<BTreeSet<i32>>,
}

impl SignalRegistry {
    pub(crate) fn new() -> Self {
        Self {
            groups: Mutex::new(BTreeSet::new()),
        }
    }

    pub(crate) fn register(self: &Arc<Self>, process_group: i32) -> GroupRegistration {
        self.groups
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(process_group);
        GroupRegistration {
            registry: Arc::clone(self),
            process_group,
        }
    }

    pub(crate) fn snapshot(&self) -> Vec<i32> {
        self.groups
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .copied()
            .collect()
    }
}

pub(crate) struct GroupRegistration {
    registry: Arc<SignalRegistry>,
    process_group: i32,
}

impl Drop for GroupRegistration {
    fn drop(&mut self) {
        self.registry
            .groups
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.process_group);
    }
}

fn active_registry() -> &'static Mutex<Option<Weak<SignalRegistry>>> {
    static ACTIVE: OnceLock<Mutex<Option<Weak<SignalRegistry>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

pub(crate) struct ParentSignalForwardingGuard {
    registry: Option<Arc<SignalRegistry>>,
    signal_handle: Option<signal_hook::iterator::Handle>,
    signal_thread: Option<std::thread::JoinHandle<()>>,
}

impl ParentSignalForwardingGuard {
    pub(crate) fn install(enabled: bool) -> std::io::Result<Self> {
        if !enabled {
            return Ok(Self {
                registry: None,
                signal_handle: None,
                signal_thread: None,
            });
        }
        let signals = signal_hook::iterator::Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGTERM,
        ])?;
        let signal_handle = signals.handle();
        let registry = Arc::new(SignalRegistry::new());
        *active_registry()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(&registry));
        let listener_registry = Arc::clone(&registry);
        let signal_thread = spawn_signal_listener(
            signals,
            listener_registry,
            signal_hook::low_level::emulate_default_handler,
        );
        Ok(Self {
            registry: Some(registry),
            signal_handle: Some(signal_handle),
            signal_thread: Some(signal_thread),
        })
    }
}

pub(crate) fn register_process_group(process_group: i32) -> Option<GroupRegistration> {
    active_registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .and_then(Weak::upgrade)
        .map(|registry| registry.register(process_group))
}

pub(crate) fn forward_signal(process_group: i32, signal: i32) {
    unsafe {
        nix::libc::kill(-process_group, signal);
    }
}

pub(crate) fn forward_signal_to_groups(process_groups: &[i32], signal: i32) {
    for process_group in process_groups {
        forward_signal(*process_group, signal);
    }
}

pub(crate) fn spawn_signal_listener<R>(
    mut signals: signal_hook::iterator::Signals,
    registry: Arc<SignalRegistry>,
    terminate_parent: impl FnOnce(i32) -> R + Send + 'static,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Some(signal) = signals.forever().next() {
            forward_signal_to_groups(&registry.snapshot(), signal);
            let _ = terminate_parent(signal);
        }
    })
}

impl Drop for ParentSignalForwardingGuard {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.take() {
            let mut active = active_registry()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if active
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|active| Arc::ptr_eq(&active, &registry))
            {
                *active = None;
            }
        }
        if let Some(handle) = self.signal_handle.take() {
            handle.close();
        }
        if let Some(thread) = self.signal_thread.take() {
            let _ = thread.join();
        }
    }
}
