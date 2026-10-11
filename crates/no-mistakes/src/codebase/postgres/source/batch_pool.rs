use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::OnceLock;

pub(super) fn pool() -> &'static ThreadPool {
    static POOL: OnceLock<ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        // Source-only batches must not initialize the global analysis pool:
        // its first repository invocation still needs to select `jobs`.
        let environment = std::env::var("RAYON_NUM_THREADS").ok();
        ThreadPoolBuilder::new()
            .num_threads(worker_count(environment.as_deref()))
            .thread_name(|index| format!("no-mistakes-sql-{index}"))
            .build()
            .expect("create PostgreSQL source worker pool")
    })
}

fn worker_count(environment: Option<&str>) -> usize {
    environment
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or_else(num_cpus::get)
}

#[cfg(test)]
#[path = "batch_pool_tests.rs"]
mod tests;
