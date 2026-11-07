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
    id: Option<String>,
    collect: bool,
    pub observer: Arc<InvocationObserver>,
    _scope: InvocationGuard,
}

impl DiagnosticRequestGuard {
    /// Strip the optional diagnostic control before strict command validation.
    /// The missing-option branch creates no observer, clock or registry entry.
    pub fn extract(options: &mut Value) -> Result<Option<Self>> {
        let collect = options
            .as_object_mut()
            .and_then(|object| object.remove("__collectDiagnostics"))
            .map(|value| {
                value
                    .as_bool()
                    .context("__collectDiagnostics must be a boolean")
            })
            .transpose()?
            .unwrap_or(false);
        let id = options
            .as_object_mut()
            .and_then(|object| object.remove("diagnosticsId"));
        let id = id
            .as_ref()
            .map(|id| {
                id.as_str()
                    .filter(|id| !id.is_empty())
                    .context("invalid options JSON: diagnosticsId must be a nonempty string")
            })
            .transpose()?;
        if collect {
            if let Some(input) = options
                .as_object_mut()
                .and_then(|object| object.remove("__diagnosticsInput"))
            {
                *options = input;
            }
        }
        if !collect && id.is_none() {
            return Ok(None);
        }
        let observer = InvocationObserver::with_progress(collect, id.is_some());
        if let Some(id) = id {
            let mut requests = requests()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if requests.get(id).and_then(Weak::upgrade).is_some() {
                bail!("diagnosticsId {id:?} is already active");
            }
            requests.insert(id.to_string(), Arc::downgrade(&observer));
        }
        let scope = InvocationGuard::install(Arc::clone(&observer));
        Ok(Some(Self {
            id: id.map(str::to_owned),
            collect,
            observer,
            _scope: scope,
        }))
    }
    pub(crate) fn report(&self, success: bool) -> Option<super::VerboseDiagnostics> {
        if self.collect {
            self.observer.increment(
                if success {
                    "output.renders"
                } else {
                    "output.errors"
                },
                1,
            );
            self.observer.verbose_snapshot()
        } else {
            None
        }
    }
}

impl Drop for DiagnosticRequestGuard {
    fn drop(&mut self) {
        if let Some(id) = &self.id {
            requests()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(id);
        }
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
