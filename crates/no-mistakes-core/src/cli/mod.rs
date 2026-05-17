pub mod output_format;

pub use output_format::Format;

use std::path::{Path, PathBuf};

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
