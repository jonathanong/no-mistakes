use super::{acquire_planning_artifact_lock_impl, rename_no_replace_impl};
use napi::{Env, Task};
use std::fs::File;
use std::path::PathBuf;

#[cfg(any(unix, windows))]
use super::unlock_planning_artifact_lock_impl;
#[cfg(any(unix, windows))]
use napi::CleanupEnvHook;

#[cfg(any(unix, windows))]
use std::collections::HashMap;
#[cfg(any(unix, windows))]
use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(any(unix, windows))]
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

#[cfg(any(unix, windows))]
struct SendCleanupHook(CleanupEnvHook<u32>);

// SAFETY: the hook pointer is only registered and removed on the Env's JS thread.
#[cfg(any(unix, windows))]
unsafe impl Send for SendCleanupHook {}

#[cfg(any(unix, windows))]
struct PlanningArtifactLock {
    file: File,
    hook: SendCleanupHook,
}

#[cfg(any(unix, windows))]
static PLANNING_ARTIFACT_LOCKS: OnceLock<Mutex<HashMap<u32, PlanningArtifactLock>>> =
    OnceLock::new();
#[cfg(any(unix, windows))]
static NEXT_PLANNING_ARTIFACT_LOCK: AtomicU32 = AtomicU32::new(1);

#[cfg(any(unix, windows))]
fn planning_artifact_locks() -> &'static Mutex<HashMap<u32, PlanningArtifactLock>> {
    PLANNING_ARTIFACT_LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(any(unix, windows))]
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

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        #[cfg(any(unix, windows))]
        {
            let locks = planning_artifact_locks().lock().unwrap();
            loop {
                let token = NEXT_PLANNING_ARTIFACT_LOCK.fetch_add(1, Ordering::Relaxed);
                if token != 0 && !locks.contains_key(&token) {
                    drop(locks);
                    let hook =
                        match _env.add_env_cleanup_hook(token, cleanup_planning_artifact_lock) {
                            Ok(hook) => hook,
                            Err(error) => {
                                drop(output);
                                return Err(error);
                            }
                        };
                    planning_artifact_locks().lock().unwrap().insert(
                        token,
                        PlanningArtifactLock {
                            file: output,
                            hook: SendCleanupHook(hook),
                        },
                    );
                    return Ok(token);
                }
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            drop(output);
            Err(napi::Error::from_reason(
                "planning artifact locks are unavailable on this platform",
            ))
        }
    }
}

pub struct ReleasePlanningArtifactLockTask {
    #[cfg(any(unix, windows))]
    token: u32,
    #[cfg(any(unix, windows))]
    hook: Option<SendCleanupHook>,
}

impl ReleasePlanningArtifactLockTask {
    pub(crate) fn new(_token: u32) -> Self {
        Self {
            #[cfg(any(unix, windows))]
            token: _token,
            #[cfg(any(unix, windows))]
            hook: None,
        }
    }
}

impl Task for ReleasePlanningArtifactLockTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> napi::Result<Self::Output> {
        #[cfg(any(unix, windows))]
        {
            let PlanningArtifactLock { file, hook } = planning_artifact_locks()
                .lock()
                .unwrap()
                .remove(&self.token)
                .ok_or_else(|| napi::Error::from_reason("unknown planning artifact lock"))?;
            self.hook = Some(hook);
            unlock_planning_artifact_lock_impl(&file)
                .map_err(|error| napi::Error::from_reason(error.to_string()))?;
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        Err(napi::Error::from_reason(
            "planning artifact locks are unavailable on this platform",
        ))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        #[cfg(any(unix, windows))]
        if let Some(hook) = self.hook.take() {
            let _ = _env.remove_env_cleanup_hook(hook.0);
        }
        Ok(output)
    }
}
