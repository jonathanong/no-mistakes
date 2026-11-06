use super::AnalysisSession;
use crate::codebase::check_outcome::CheckFailure;
use dashmap::DashMap;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

mod vitest_project_policy;

#[derive(Default)]
pub(super) struct CheckOutcomes {
    partial: AtomicBool,
    vitest_project_policy: std::sync::OnceLock<
        std::sync::Arc<crate::codebase::rules::vitest_project_policy::Diagnostics>,
    >,
    demands: DashMap<String, BTreeSet<PathBuf>>,
    failures: DashMap<CheckFailure, ()>,
    source_failures: DashMap<PathBuf, String>,
    prerequisites: DashMap<String, crate::codebase::check_outcome::Prerequisite<()>>,
}

impl AnalysisSession {
    pub fn scoped_checks(&self) -> std::sync::Arc<Self> {
        let scoped = std::sync::Arc::new(Self {
            resources: std::sync::Arc::clone(&self.resources),
            check_outcomes: CheckOutcomes::default(),
        });
        if self.partial_checks_enabled() {
            scoped.enable_partial_checks();
        }
        scoped
    }

    pub fn set_check_prerequisite(
        &self,
        name: &str,
        state: crate::codebase::check_outcome::Prerequisite<()>,
    ) {
        if let crate::codebase::check_outcome::Prerequisite::Failed(failures) = &state {
            for failure in failures {
                self.record_check_failure(failure.clone());
            }
        }
        self.check_outcomes
            .prerequisites
            .insert(name.to_owned(), state);
    }

    pub fn check_prerequisite(
        &self,
        name: &str,
    ) -> Option<crate::codebase::check_outcome::Prerequisite<()>> {
        self.check_outcomes
            .prerequisites
            .get(name)
            .map(|state| state.clone())
    }

    pub fn enable_partial_checks(&self) {
        self.check_outcomes.partial.store(true, Ordering::Relaxed);
    }

    pub fn partial_checks_enabled(&self) -> bool {
        self.check_outcomes.partial.load(Ordering::Relaxed)
    }

    pub fn register_check_demand(&self, consumer: &str, files: impl IntoIterator<Item = PathBuf>) {
        let files: BTreeSet<_> = files.into_iter().collect();
        self.check_outcomes
            .demands
            .entry(consumer.to_owned())
            .or_default()
            .extend(files.iter().cloned());
        for file in &files {
            if let Some(message) = self.check_outcomes.source_failures.get(file) {
                self.record_check_failure(CheckFailure {
                    consumer: consumer.to_owned(),
                    file: Some(file.clone()),
                    property: None,
                    message: message.clone(),
                });
            }
        }
    }

    pub fn record_check_failure(&self, failure: CheckFailure) {
        self.check_outcomes.failures.insert(failure, ());
    }

    pub fn check_failures(&self) -> Vec<CheckFailure> {
        let mut failures: Vec<_> = self
            .check_outcomes
            .failures
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        failures.sort();
        failures
    }

    pub fn check_consumer_ready(&self, consumer: &str) -> bool {
        !self
            .check_outcomes
            .failures
            .iter()
            .any(|entry| entry.key().consumer == consumer)
    }

    pub fn record_check_source_failure(&self, file: &std::path::Path, message: &str) {
        self.check_outcomes
            .source_failures
            .insert(file.to_owned(), message.to_owned());
        for demand in &self.check_outcomes.demands {
            if demand.value().contains(file) {
                self.record_check_failure(CheckFailure {
                    consumer: demand.key().clone(),
                    file: Some(file.to_owned()),
                    property: None,
                    message: message.to_owned(),
                });
            }
        }
    }

    pub fn capture_check_result<T>(
        &self,
        consumer: &str,
        result: anyhow::Result<T>,
    ) -> anyhow::Result<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) if self.partial_checks_enabled() => {
                let input =
                    error.downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>();
                self.record_check_failure(CheckFailure {
                    consumer: consumer.to_owned(),
                    file: input.and_then(|input| input.file.clone()),
                    property: input.and_then(|input| input.property.clone()),
                    message: format!("{error:#}"),
                });
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests;
