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
}

impl RequestCache {
    fn new(fact_plan: Option<RunnerConfigFactPlan>) -> Self {
        Self {
            programs: crate::ast::current_request_parse_cache().unwrap_or_default(),
            fact_plan,
            helper_fact_paths: HashSet::new(),
            helper_facts: HashMap::new(),
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
        REQUEST_CACHES.with(|caches| caches.borrow_mut().push(RequestCache::new(fact_plan)));
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
            caches.push(RequestCache::new(None));
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
        collect_helper_facts(path, program, source);
        analyze(program, source)
    };
    with_request_program(path, source, analyze_program)
}

fn collect_helper_facts(path: &Path, program: &oxc_ast::ast::Program<'_>, source: &str) {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let request = REQUEST_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let request = caches.last_mut()?;
        if !request.helper_fact_paths.insert(path.clone()) {
            return None;
        }
        request.fact_plan.clone()
    });
    let Some(request) = request else {
        return;
    };
    let (plan, playwright) = if request.primary_files.contains(&path) {
        (&request.primary_plan, request.playwright.as_ref())
    } else if request.graph_files.contains(&path) {
        (&request.graph_plan, request.playwright.as_ref())
    } else {
        return;
    };
    if plan
        .integration_runner_configs
        .as_ref()
        .is_some_and(|configs| configs.contains(&path))
    {
        return;
    }
    let facts = crate::codebase::check_facts::collect_file_facts_from_program(
        &request.root,
        &path,
        plan,
        playwright,
        source,
        program,
    );
    REQUEST_CACHES.with(|caches| {
        if let Some(cache) = caches.borrow_mut().last_mut() {
            cache.helper_facts.insert(path, facts);
        }
    });
}
