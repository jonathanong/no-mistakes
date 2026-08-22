use super::*;
use std::cell::Cell;

impl AnalysisSession {
    /// Parse strictly through the request OXC gateway and return only callback-
    /// owned data. Failed parses retain the stable `ast::with_program` error.
    pub(crate) fn with_program<T>(
        &self,
        path: &Path,
        source: &Arc<str>,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_program_observed(
            &path,
            Arc::clone(source),
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            analyze,
        );
        self.record_parse_error_if(parse_started.get(), result.is_err());
        result
    }

    /// Parse through the thread-local OXC gateway and return only callback-
    /// owned data. The source may be a recovered program with a diagnostic.
    pub(crate) fn with_recovered_program<T>(
        &self,
        path: &Path,
        source: &Arc<str>,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str, Option<String>) -> T,
    ) -> anyhow::Result<T> {
        self.with_recovered_program_status(path, source, |program, source, diagnostic, _| {
            analyze(program, source, diagnostic)
        })
    }

    /// Recovered parser access with an explicit fatal-panic marker for fact
    /// collectors that expose partial AST-derived results to sound queries.
    pub(crate) fn with_recovered_program_status<T>(
        &self,
        path: &Path,
        source: &Arc<str>,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str, Option<String>, bool) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_recovered_program_status_observed(
            &path,
            Arc::clone(source),
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            |program, source, parse_error, panicked| {
                self.record_parse_error_if(parse_started.get(), parse_error.is_some());
                analyze(program, source, parse_error, panicked)
            },
        );
        self.record_parse_error_if(parse_started.get(), result.is_err());
        result
    }

    /// Parse unknown extensions as TypeScript while retaining recovered
    /// diagnostics. This preserves the fact collector's direct-file fallback.
    pub(crate) fn with_recovered_typescript_program<T>(
        &self,
        path: &Path,
        source: &Arc<str>,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str, Option<String>) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_recovered_typescript_program_observed(
            &path,
            Arc::clone(source),
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            |program, source, parse_error| {
                self.record_parse_error_if(parse_started.get(), parse_error.is_some());
                analyze(program, source, parse_error)
            },
        );
        self.record_parse_error_if(parse_started.get(), result.is_err());
        result
    }

    /// Parse with the historical symbols source type while retaining recovered
    /// diagnostics for generic fact consumers. Only parser panics are fatal.
    pub(crate) fn with_legacy_symbols_program<T>(
        &self,
        path: &Path,
        source: &Arc<str>,
        analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str, Option<String>) -> T,
    ) -> anyhow::Result<T> {
        let path = normalize_path(path);
        self.increment("parse.requests", 1);
        let parse_started = Cell::new(false);
        let result = crate::ast::with_legacy_symbols_program_observed(
            &path,
            Arc::clone(source),
            || {
                parse_started.set(true);
                self.record_parse(&path);
            },
            |program, source, parse_error| {
                self.record_parse_error_if(parse_started.get(), parse_error.is_some());
                analyze(program, source, parse_error)
            },
        );
        self.record_parse_error_if(parse_started.get(), result.is_err());
        result
    }

    pub fn work_snapshot(&self) -> SessionWorkSnapshot {
        SessionWorkSnapshot {
            source_reads: self
                .observer
                .as_ref()
                .map(|observer| observer.source_read_snapshot())
                .unwrap_or_default(),
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

    fn record_parse_error_if(&self, started: bool, failed: bool) {
        if started && failed {
            self.increment("parse.errors", 1);
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
