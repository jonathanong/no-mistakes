use super::{
    acquire_planning_artifact_lock_impl, rename_no_replace_impl, unlock_planning_artifact_lock_impl,
};
use napi::{Env, Task};
use std::fs::File;
use std::path::PathBuf;

#[cfg(unix)]
use std::collections::HashMap;
#[cfg(unix)]
use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(unix)]
use std::sync::{Mutex, OnceLock};

pub struct RenameNoReplaceTask {
    from: PathBuf,
    to: PathBuf,
}

impl RenameNoReplaceTask {
    pub(crate) fn new(from: PathBuf, to: PathBuf) -> Self {
        Self { from, to }
    }
}

impl Task for RenameNoReplaceTask {
    type Output = bool;
    type JsValue = bool;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        rename_no_replace_impl(&self.from, &self.to)
            .map_err(|error| napi::Error::from_reason(error.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg(unix)]
static PLANNING_ARTIFACT_LOCKS: OnceLock<Mutex<HashMap<u32, File>>> = OnceLock::new();
#[cfg(unix)]
static NEXT_PLANNING_ARTIFACT_LOCK: AtomicU32 = AtomicU32::new(1);

#[cfg(unix)]
fn planning_artifact_locks() -> &'static Mutex<HashMap<u32, File>> {
    PLANNING_ARTIFACT_LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(unix)]
fn cleanup_planning_artifact_lock(token: u32) {
    if let Ok(mut locks) = planning_artifact_locks().lock() {
        locks.remove(&token);
    }
}

pub struct AcquirePlanningArtifactLockTask {
    path: PathBuf,
}

impl AcquirePlanningArtifactLockTask {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Task for AcquirePlanningArtifactLockTask {
    type Output = File;
    type JsValue = u32;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        acquire_planning_artifact_lock_impl(&self.path)
            .map_err(|error| napi::Error::from_reason(error.to_string()))
    }

    fn resolve(&mut self, env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        #[cfg(unix)]
        {
            let mut locks = planning_artifact_locks().lock().unwrap();
            loop {
                let token = NEXT_PLANNING_ARTIFACT_LOCK.fetch_add(1, Ordering::Relaxed);
                if token != 0 && !locks.contains_key(&token) {
                    locks.insert(token, output);
                    drop(locks);
                    if let Err(error) =
                        env.add_env_cleanup_hook(token, cleanup_planning_artifact_lock)
                    {
                        cleanup_planning_artifact_lock(token);
                        return Err(error);
                    }
                    return Ok(token);
                }
            }
        }
        #[cfg(not(unix))]
        {
            drop(output);
            Err(napi::Error::from_reason(
                "planning artifact locks are unavailable on this platform",
            ))
        }
    }
}

pub struct ReleasePlanningArtifactLockTask {
    token: u32,
}

impl ReleasePlanningArtifactLockTask {
    pub(crate) fn new(token: u32) -> Self {
        Self { token }
    }
}

impl Task for ReleasePlanningArtifactLockTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> napi::Result<Self::Output> {
        #[cfg(unix)]
        {
            let file = planning_artifact_locks()
                .lock()
                .unwrap()
                .remove(&self.token)
                .ok_or_else(|| napi::Error::from_reason("unknown planning artifact lock"))?;
            unlock_planning_artifact_lock_impl(&file)
                .map_err(|error| napi::Error::from_reason(error.to_string()))?;
            Ok(())
        }
        #[cfg(not(unix))]
        Err(napi::Error::from_reason(
            "planning artifact locks are unavailable on this platform",
        ))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}
