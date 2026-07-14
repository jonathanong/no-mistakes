pub mod output_format;
mod traversal;
mod traversal_impls;

pub use output_format::{resolve_format, Format};
pub(crate) use traversal::related_edge_view;
pub use traversal::{edge_view, TraversableEdge};

use std::path::{Path, PathBuf};

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

pub fn init_rayon_threads(args: JobsArg) {
    let raw_threads = std::env::var("RAYON_NUM_THREADS").ok();
    let threads = rayon_thread_count(args, raw_threads.as_deref());
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global();
}

fn rayon_thread_count(args: JobsArg, raw_threads: Option<&str>) -> usize {
    if args.jobs > 0 {
        args.jobs
    } else if let Some(raw) = raw_threads {
        raw.parse().unwrap_or_else(|_| num_cpus::get())
    } else {
        num_cpus::get()
    }
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
