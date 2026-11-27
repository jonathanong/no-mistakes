use rayon::ThreadPoolBuilder;
use std::sync::OnceLock;

static RECORDED_RAYON_THREADS: OnceLock<usize> = OnceLock::new();

/// Record the process-wide Rayon pool size chosen by the host binary.
pub fn record_global_rayon_threads(threads: usize) {
    let _ = RECORDED_RAYON_THREADS.set(threads);
}

pub(super) fn install<T: Send>(jobs: usize, work: impl FnOnce() -> T + Send) -> T {
    let environment = std::env::var("RAYON_NUM_THREADS").ok();
    let recorded = RECORDED_RAYON_THREADS.get().copied();
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
    let requested = rayon_thread_count(jobs, environment);
    recorded.map_or(requested, |recorded| requested.min(recorded))
}

fn rayon_thread_count(jobs: usize, raw_threads: Option<&str>) -> usize {
    if jobs > 0 {
        return jobs;
    }
    raw_threads
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or_else(num_cpus::get)
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1, nm_shard_postgres_source)))]
#[path = "batch_pool_tests.rs"]
mod tests;
