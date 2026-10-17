use rayon::ThreadPoolBuilder;

use crate::cli::{rayon_thread_count, recorded_rayon_threads, JobsArg};

pub(super) fn install<T: Send>(jobs: usize, work: impl FnOnce() -> T + Send) -> T {
    let environment = std::env::var("RAYON_NUM_THREADS").ok();
    let recorded = recorded_rayon_threads();
    let threads = worker_limit(jobs, environment.as_deref(), recorded);
    // Source-only batches must not initialize the global analysis pool:
    // its first repository invocation still needs to select `jobs`.
    // Reuse that pool when it is already this size; otherwise build a local
    // pool no larger than the recorded analysis pool.
    dispatch(threads, recorded, work)
}

pub(super) fn worker_limit(
    jobs: usize,
    environment: Option<&str>,
    recorded: Option<usize>,
) -> usize {
    let requested = rayon_thread_count(JobsArg { jobs }, environment);
    recorded.map_or(requested, |recorded| requested.min(recorded))
}

fn dispatch<T: Send>(
    threads: usize,
    recorded: Option<usize>,
    work: impl FnOnce() -> T + Send,
) -> T {
    if recorded == Some(threads) {
        return work();
    }
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|index| format!("no-mistakes-sql-{index}"))
        .build()
        .expect("create PostgreSQL source worker pool")
        .install(work)
}

#[cfg(test)]
#[path = "batch_pool_tests.rs"]
mod tests;
