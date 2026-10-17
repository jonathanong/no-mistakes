mod json_write;
pub mod output_format;
mod traversal;
mod traversal_impls;

pub(crate) use json_write::json_value;
pub use json_write::{json_pretty, json_string, print_json, yaml_string};
pub use output_format::{resolve_format, Format};
pub use traversal::{edge_view, TraversableEdge};
pub(crate) use traversal::{
    prepared_edge_view, prepared_related_view, project_relationship_edge, related_edge_view,
};

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Resolve edge traversal depth for commands that optionally start from roots.
///
/// With no roots, `None` means the full edge list. With roots and no explicit
/// depth, the default is direct edges only (`Some(1)`).
pub fn root_scoped_edge_depth<T>(roots: &[T], depth: Option<usize>) -> Option<usize> {
    if roots.is_empty() {
        depth
    } else {
        Some(depth.unwrap_or(1))
    }
}

#[derive(clap::Args, Debug, Clone, Copy, Default)]
pub struct JobsArg {
    #[arg(
        short = 'j',
        long = "jobs",
        value_name = "N",
        default_value_t = 0,
        global = true
    )]
    pub jobs: usize,
}

static RECORDED_RAYON_THREADS: OnceLock<usize> = OnceLock::new();

/// Warning when a later request cannot resize the process-wide Rayon pool.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobsPoolWarning {
    pub code: &'static str,
    pub message: String,
    pub requested_jobs: usize,
    pub effective_jobs: usize,
}

/// Builds the global pool on the first successful attempt and never resizes it.
///
/// Returns a warning when this call's resolved count differs from the recorded
/// size. A successful first install records that count and does not warn.
pub fn init_rayon_threads(args: JobsArg) -> Option<JobsPoolWarning> {
    let environment = std::env::var("RAYON_NUM_THREADS").ok();
    let requested = rayon_thread_count(args, environment.as_deref());
    let recorded = *RECORDED_RAYON_THREADS.get_or_init(|| install_global_pool(requested));
    jobs_pool_mismatch(requested, Some(recorded))
}

pub fn init_rayon_threads_if_requested(jobs: Option<usize>) -> Option<JobsPoolWarning> {
    init_rayon_threads(JobsArg {
        jobs: jobs.unwrap_or(0),
    })
}

pub(crate) fn recorded_rayon_threads() -> Option<usize> {
    RECORDED_RAYON_THREADS.get().copied()
}

pub(crate) fn rayon_thread_count(args: JobsArg, raw_threads: Option<&str>) -> usize {
    if args.jobs > 0 {
        return args.jobs;
    }
    raw_threads
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or_else(num_cpus::get)
}

/// `None` means this resolution is the initializing call, which does not warn.
pub(crate) fn jobs_pool_mismatch(
    requested: usize,
    recorded: Option<usize>,
) -> Option<JobsPoolWarning> {
    let effective = recorded.filter(|effective| *effective != requested)?;
    Some(JobsPoolWarning {
        code: "rayon-pool-size",
        message: format!(
            "resolved {requested} Rayon workers but the process pool has {effective} and cannot be resized"
        ),
        requested_jobs: requested,
        effective_jobs: effective,
    })
}

pub(crate) fn attach_jobs_pool_warning(json: String, warning: Option<JobsPoolWarning>) -> String {
    let Some(warning) = warning else {
        return json;
    };
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&json) else {
        return json;
    };
    let Some(object) = value.as_object_mut() else {
        return json;
    };
    object.insert(
        "jobsPoolWarning".to_string(),
        serde_json::to_value(warning).expect("jobs pool warning serializes"),
    );
    value.to_string()
}

fn pool_size_after_build(requested: usize, error: Option<&str>) -> usize {
    match error {
        None => requested,
        Some(message) if message.contains("already been initialized") => {
            rayon::current_num_threads()
        }
        Some(_) => requested,
    }
}

fn install_global_pool(requested: usize) -> usize {
    let error = rayon::ThreadPoolBuilder::new()
        .num_threads(requested)
        .build_global()
        .err()
        .map(|error| error.to_string());
    pool_size_after_build(requested, error.as_deref())
}

pub fn resolve_root(root: &Path, cwd: &Path) -> PathBuf {
    if root.is_absolute() {
        root.to_path_buf()
    } else {
        cwd.join(root)
    }
}

pub fn resolve_optional_root(root: Option<&Path>, cwd: &Path) -> PathBuf {
    root.map(|root| resolve_root(root, cwd))
        .unwrap_or_else(|| cwd.to_path_buf())
}

#[cfg(test)]
mod tests;
