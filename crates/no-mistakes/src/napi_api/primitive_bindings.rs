#[cfg(not(coverage))]
#[cfg_attr(not(test), napi)]
pub fn version() -> AsyncTask<VersionTask> {
    AsyncTask::new(VersionTask)
}

pub(crate) fn version_impl() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Internal async primitive used by the JavaScript artifact publisher.
#[cfg(not(coverage))]
#[cfg_attr(not(test), napi)]
pub fn rename_no_replace(from: String, to: String) -> AsyncTask<filesystem::RenameNoReplaceTask> {
    AsyncTask::new(filesystem::RenameNoReplaceTask::new(from.into(), to.into()))
}

/// Internal async primitive that holds an interprocess artifact lock.
#[cfg(not(coverage))]
#[cfg_attr(not(test), napi)]
pub fn acquire_planning_artifact_lock(
    path: String,
) -> AsyncTask<filesystem::AcquirePlanningArtifactLockTask> {
    AsyncTask::new(filesystem::AcquirePlanningArtifactLockTask::new(
        path.into(),
    ))
}

/// Internal async primitive that releases an interprocess artifact lock.
#[cfg(not(coverage))]
#[cfg_attr(not(test), napi)]
pub fn release_planning_artifact_lock(
    token: u32,
) -> AsyncTask<filesystem::ReleasePlanningArtifactLockTask> {
    AsyncTask::new(filesystem::ReleasePlanningArtifactLockTask::new(token))
}
