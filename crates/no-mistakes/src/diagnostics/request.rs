//! Live request lookup only. Weak references never retain completed analysis.
use super::{InvocationGuard, InvocationObserver, ProgressSnapshot};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock, Weak};

fn requests() -> &'static Mutex<BTreeMap<String, Weak<InvocationObserver>>> {
    static REQUESTS: OnceLock<Mutex<BTreeMap<String, Weak<InvocationObserver>>>> = OnceLock::new();
    REQUESTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub struct DiagnosticRequestGuard {
    id: String,
    pub observer: Arc<InvocationObserver>,
    _scope: InvocationGuard,
}

impl DiagnosticRequestGuard {
    /// Strip the optional diagnostic control before strict command validation.
    /// The missing-option branch creates no observer, clock or registry entry.
    pub fn extract(options: &mut Value) -> Result<Option<Self>> {
        let Some(id) = options
            .as_object_mut()
            .and_then(|object| object.remove("diagnosticsId"))
        else {
            return Ok(None);
        };
        let id = id
            .as_str()
            .filter(|id| !id.is_empty())
            .context("invalid options JSON: diagnosticsId must be a nonempty string")?;
        let mut requests = requests()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if requests.get(id).and_then(Weak::upgrade).is_some() {
            bail!("diagnosticsId {id:?} is already active");
        }
        let observer = InvocationObserver::with_progress(false, true);
        requests.insert(id.to_string(), Arc::downgrade(&observer));
        let scope = InvocationGuard::install(Arc::clone(&observer));
        Ok(Some(Self {
            id: id.to_string(),
            observer,
            _scope: scope,
        }))
    }
}

impl Drop for DiagnosticRequestGuard {
    fn drop(&mut self) {
        requests()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.id);
    }
}

pub fn progress_for_request(id: &str) -> Option<ProgressSnapshot> {
    let observer = requests()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(id)
        .and_then(Weak::upgrade)?;
    observer.progress_snapshot()
}

#[cfg(test)]
mod tests;
