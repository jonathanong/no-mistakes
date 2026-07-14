use super::*;
use std::cell::Cell;

impl AnalysisSession {
    /// Parse strictly through the request OXC gateway and return only callback-
    /// owned data. Failed parses retain the stable `ast::with_program` error.
    pub(crate) fn with_program<T>(
        &self,
        path: &Path,
        source: &str,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_program_observed(
            &path,
            source,
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            analyze,
        );
        if parse_started.get() && result.is_err() {
            self.increment("parse.errors", 1);
        }
        result
    }

    /// Parse through the thread-local OXC gateway and return only callback-
    /// owned data. The source may be a recovered program with a diagnostic.
    pub(crate) fn with_recovered_program<T>(
        &self,
        path: &Path,
        source: &str,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str, Option<String>) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_recovered_program_observed(
            &path,
            source,
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            |program, source, parse_error| {
                if parse_started.get() && parse_error.is_some() {
                    self.increment("parse.errors", 1);
                }
                analyze(program, source, parse_error)
            },
        );
        if parse_started.get() && result.is_err() {
            self.increment("parse.errors", 1);
        }
        result
    }

    /// Parse through the recovered-program gateway while preserving the TS
    /// fact collector's explicit unknown-extension fallback.
    pub(crate) fn with_recovered_typescript_program<T>(
        &self,
        path: &Path,
        source: &str,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str, Option<String>) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_recovered_typescript_program_observed(
            &path,
            source,
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            |program, source, parse_error| {
                if parse_started.get() && parse_error.is_some() {
                    self.increment("parse.errors", 1);
                }
                analyze(program, source, parse_error)
            },
        );
        if parse_started.get() && result.is_err() {
            self.increment("parse.errors", 1);
        }
        result
    }

    pub fn work_snapshot(&self) -> SessionWorkSnapshot {
        SessionWorkSnapshot {
            source_reads: BTreeMap::new(),
            parse_attempts: snapshot_map(self.parse_attempts.as_ref()),
        }
    }

    pub(crate) fn record_work(&self, metric: &'static str, amount: u64) {
        self.increment(metric, amount);
    }

    fn record_parse(&self, path: &Path) {
        self.increment("parse.files", 1);
        if let Some(attempts) = &self.parse_attempts {
            *attempts.entry(path.to_path_buf()).or_default() += 1;
        }
    }

    pub(super) fn increment(&self, metric: &'static str, amount: u64) {
        if let Some(observer) = &self.observer {
            observer.increment(metric, amount);
        }
    }
}

fn snapshot_map(map: Option<&DashMap<PathBuf, u64>>) -> BTreeMap<PathBuf, u64> {
    map.into_iter()
        .flat_map(|map| {
            map.iter()
                .map(|entry| (entry.key().clone(), *entry.value()))
        })
        .collect()
}
