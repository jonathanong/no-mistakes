use super::{FileAnalysis, PreparedIntegrationRunnerConfigs, RunnerConfigFactPlan};
use crate::ast::ParsedProgramCache;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

struct AnalysisCollector {
    analyses: BTreeMap<PathBuf, FileAnalysis>,
}

struct RequestCache {
    programs: ParsedProgramCache,
    fact_plan: Option<RunnerConfigFactPlan>,
    helper_fact_paths: HashSet<PathBuf>,
    helper_facts: HashMap<PathBuf, crate::codebase::check_facts::CheckFileFacts>,
    sources: Option<std::sync::Arc<crate::codebase::ts_source::SourceStore>>,
}

impl RequestCache {
    fn new(
        fact_plan: Option<RunnerConfigFactPlan>,
        sources: Option<std::sync::Arc<crate::codebase::ts_source::SourceStore>>,
    ) -> Self {
        Self {
            programs: crate::ast::current_request_parse_cache().unwrap_or_default(),
            fact_plan,
            helper_fact_paths: HashSet::new(),
            helper_facts: HashMap::new(),
            sources,
        }
    }
}

thread_local! {
    static ANALYSIS_COLLECTORS: RefCell<Vec<AnalysisCollector>> =
        const { RefCell::new(Vec::new()) };
    static REQUEST_CACHES: RefCell<Vec<RequestCache>> = const { RefCell::new(Vec::new()) };
}

impl PreparedIntegrationRunnerConfigs {
    pub(crate) fn with_request_cache<T>(
        &self,
        fact_plan: Option<RunnerConfigFactPlan>,
        collect: impl FnOnce() -> T,
    ) -> (
        T,
        HashMap<PathBuf, crate::codebase::check_facts::CheckFileFacts>,
    ) {
        self.with_request_cache_and_sources(fact_plan, None, collect)
    }

    pub(crate) fn with_request_cache_and_sources<T>(
        &self,
        fact_plan: Option<RunnerConfigFactPlan>,
        sources: Option<std::sync::Arc<crate::codebase::ts_source::SourceStore>>,
        collect: impl FnOnce() -> T,
    ) -> (
        T,
        HashMap<PathBuf, crate::codebase::check_facts::CheckFileFacts>,
    ) {
        REQUEST_CACHES.with(|caches| {
            caches.borrow_mut().push(RequestCache::new(
                fact_plan,
                sources.or_else(|| self.sources.clone()),
            ))
        });
        let result = collect();
        let helper_facts = REQUEST_CACHES.with(|caches| {
            caches
                .borrow_mut()
                .pop()
                .expect("runner-config request cache must be active")
                .helper_facts
        });
        (result, helper_facts)
    }
}

pub(super) fn collect_analyses<T>(
    collect: impl FnOnce() -> T,
) -> (T, BTreeMap<PathBuf, FileAnalysis>) {
    let owns_request_cache = REQUEST_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        if caches.is_empty() {
            caches.push(RequestCache::new(None, None));
            true
        } else {
            false
        }
    });
    ANALYSIS_COLLECTORS.with(|collectors| {
        collectors.borrow_mut().push(AnalysisCollector {
            analyses: BTreeMap::new(),
        });
    });
    let result = collect();
    let analyses = ANALYSIS_COLLECTORS.with(|collectors| {
        collectors
            .borrow_mut()
            .pop()
            .expect("runner-config analysis collector must be active")
            .analyses
    });
    if owns_request_cache {
        REQUEST_CACHES.with(|caches| {
            caches
                .borrow_mut()
                .pop()
                .expect("temporary runner-config request cache must be active");
        });
    }
    (result, analyses)
}

pub(in crate::integration_tests) fn read_request_source(
    path: &Path,
) -> Result<std::sync::Arc<str>> {
    let sources = REQUEST_CACHES.with(|caches| {
        caches
            .borrow()
            .last()
            .and_then(|request| request.sources.clone())
    });
    match sources {
        Some(sources) => sources
            .read_path(path)
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
        None => std::fs::read_to_string(path)
            .map(std::sync::Arc::<str>::from)
            .map_err(anyhow::Error::from),
    }
}

pub(super) fn with_request_program<T>(
    path: &Path,
    source: &str,
    analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str) -> T,
) -> Result<T> {
    let cache = REQUEST_CACHES.with(|caches| {
        caches
            .borrow()
            .last()
            .map(|request| request.programs.clone())
    });
    match cache {
        Some(cache) => cache
            .with_program(path, source, analyze)
            .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display())),
        None => crate::ast::with_program(path, source, analyze),
    }
}

pub(in crate::integration_tests) fn with_program<T>(
    path: &Path,
    source: &str,
    analyze: impl for<'a> FnOnce(&'a oxc_ast::ast::Program<'a>, &'a str) -> T,
) -> Result<T> {
    let analyze_program = |program: &oxc_ast::ast::Program<'_>, source: &str| {
        ANALYSIS_COLLECTORS.with(|collectors| {
            if let Some(collector) = collectors.borrow_mut().last_mut() {
                collector.analyses.insert(
                    crate::codebase::ts_resolver::normalize_path(path),
                    crate::integration_tests::analysis::analyze_program(path, program, source),
                );
            }
        });
        helper_facts::collect_helper_facts(path, program, source);
        analyze(program, source)
    };
    with_request_program(path, source, analyze_program)
}

mod helper_facts;
