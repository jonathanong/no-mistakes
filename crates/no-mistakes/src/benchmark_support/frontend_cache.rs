//! Benchmark-only cold/warm boundaries; no production collector is replaced.
use super::{collect_language_frontend_facts, LanguageFrontendFixture, LanguageFrontendSummary};

pub struct FrontendCacheLifecycle {
    fixture: LanguageFrontendFixture,
    pool: rayon::ThreadPool,
    pub cold_collections: usize,
    pub broadcast_workers: Vec<usize>,
    pub warm_collections: usize,
}

impl FrontendCacheLifecycle {
    pub fn new(fixture: LanguageFrontendFixture) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        // Start every worker before the boundary without touching a frontend.
        assert_eq!(pool.broadcast(|ctx| ctx.index()), vec![0, 1, 2, 3]);
        Self {
            fixture,
            pool,
            cold_collections: 0,
            broadcast_workers: Vec::new(),
            warm_collections: 0,
        }
    }

    pub fn cold(&mut self) -> LanguageFrontendSummary {
        assert_eq!(
            self.cold_collections, 0,
            "cold collection must execute only once per process"
        );
        assert!(self.broadcast_workers.is_empty());
        self.cold_collections += 1;
        self.pool
            .install(|| collect_language_frontend_facts(&self.fixture))
    }

    pub fn exercise_workers(&mut self) -> Vec<LanguageFrontendSummary> {
        assert!(self.broadcast_workers.is_empty());
        let results = self
            .pool
            .broadcast(|ctx| (ctx.index(), collect_language_frontend_facts(&self.fixture)));
        self.broadcast_workers = results.iter().map(|(worker, _)| *worker).collect();
        assert_eq!(self.broadcast_workers, vec![0, 1, 2, 3]);
        results.into_iter().map(|(_, summary)| summary).collect()
    }

    pub fn warm(&mut self) -> LanguageFrontendSummary {
        assert_eq!(self.broadcast_workers, vec![0, 1, 2, 3]);
        self.warm_collections += 1;
        self.pool
            .install(|| collect_language_frontend_facts(&self.fixture))
    }
}

#[cfg(test)]
#[path = "frontend_cache/tests.rs"]
mod tests;
