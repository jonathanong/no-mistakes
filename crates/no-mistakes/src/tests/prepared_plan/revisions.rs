use crate::tests::PlanArgs;
use dashmap::DashMap;
use no_mistakes::codebase::ts_source::SourceStore;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

type HistoricalSourceSlots = DashMap<(String, String), Arc<OnceLock<Option<Arc<str>>>>>;
type RevisionExistenceSlots = DashMap<String, Arc<OnceLock<bool>>>;

/// Request-scoped projection of the working tree and requested Git revisions.
///
/// Semantic dependency analyzers borrow this projection instead of independently
/// probing Git or opening files. This keeps both successful and failed
/// working-tree reads in the prepared request's `SourceStore`, while historical
/// reads are memoized by revision and repository-relative path.
pub(crate) struct RevisionSources {
    git_root: PathBuf,
    base: Option<String>,
    head: Option<String>,
    diff_only: bool,
    sources: Arc<SourceStore>,
    historical: HistoricalSourceSlots,
    existence: RevisionExistenceSlots,
    #[cfg(test)]
    ref_probe_count: std::sync::atomic::AtomicUsize,
}

impl RevisionSources {
    pub(crate) fn prepare(root: &Path, args: &PlanArgs, sources: Arc<SourceStore>) -> Self {
        Self {
            git_root: find_git_root(root).unwrap_or_else(|| root.to_path_buf()),
            base: args.base.clone(),
            head: args.head.clone(),
            diff_only: args.head.is_none()
                && super::super::changed_files::has_explicit_diff_source(args),
            sources,
            historical: DashMap::new(),
            existence: DashMap::new(),
            #[cfg(test)]
            ref_probe_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub(crate) fn base_name(&self) -> Option<&str> {
        self.base.as_deref()
    }

    pub(crate) fn read_base(&self, path: &Path) -> Option<Arc<str>> {
        self.base
            .as_deref()
            .and_then(|revision| self.read_revision(revision, path))
    }

    pub(crate) fn read_after(&self, path: &Path) -> Option<Arc<str>> {
        self.head.as_deref().map_or_else(
            || {
                (!self.diff_only)
                    .then(|| self.sources.read_path(path).ok())
                    .flatten()
            },
            |revision| self.read_revision(revision, path),
        )
    }

    pub(crate) fn read_after_or_empty(&self, path: &Path) -> Arc<str> {
        self.read_after(path).unwrap_or_else(|| Arc::from(""))
    }

    pub(crate) fn base_ref_exists(&self) -> bool {
        self.base_name()
            .is_some_and(|revision| self.ref_exists(revision))
    }

    pub(crate) fn head_name(&self) -> Option<&str> {
        self.head.as_deref()
    }

    pub(crate) fn head_ref_exists(&self) -> bool {
        self.head_name()
            .is_some_and(|revision| self.ref_exists(revision))
    }

    pub(crate) fn is_diff_only(&self) -> bool {
        self.diff_only
    }

    fn read_revision(&self, revision: &str, path: &Path) -> Option<Arc<str>> {
        let relative = path
            .strip_prefix(&self.git_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let key = (revision.to_string(), relative.clone());
        let slot = if let Some(existing) = self.historical.get(&key) {
            Arc::clone(existing.value())
        } else {
            Arc::clone(
                self.historical
                    .entry(key)
                    .or_insert_with(|| Arc::new(OnceLock::new()))
                    .value(),
            )
        };
        slot.get_or_init(|| git_show_file(&self.git_root, revision, &relative).map(Arc::from))
            .clone()
    }

    fn ref_exists(&self, revision: &str) -> bool {
        let slot = Arc::clone(
            self.existence
                .entry(revision.to_string())
                .or_insert_with(|| Arc::new(OnceLock::new()))
                .value(),
        );
        *slot.get_or_init(|| {
            #[cfg(test)]
            self.ref_probe_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            git_ref_exists(&self.git_root, revision)
        })
    }

    #[cfg(test)]
    fn ref_probe_count(&self) -> usize {
        self.ref_probe_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

fn find_git_root(dir: &Path) -> Option<PathBuf> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir);
    let output = crate::invocation::command_output(&mut command).ok()?;
    output.status.success().then(|| {
        String::from_utf8(output.stdout)
            .ok()
            .map(|value| PathBuf::from(value.trim()))
    })?
}

fn git_ref_exists(root: &Path, revision: &str) -> bool {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--verify", revision])
        .current_dir(root);
    crate::invocation::command_output(&mut command).is_ok_and(|output| output.status.success())
}

fn git_show_file(root: &Path, revision: &str, relative: &str) -> Option<String> {
    let mut command = std::process::Command::new("git");
    command
        .args(["show", &format!("{revision}:{relative}")])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command).ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())?
}

#[cfg(test)]
#[path = "revisions/tests.rs"]
mod tests;
