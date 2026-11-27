use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_queues::factory::find_queue_factory_facts_from_program;
use crate::codebase::ts_queues::usage::extract_queue_usage_from_program;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{TsFactContext, TsFactPlan};
use crate::codebase::ts_source::SourceStore;
use crate::queue_analysis_hooks::{QueueAnalysisHooks, QueueFileFacts};
use oxc_ast::ast::Program;
use oxc_semantic::Scoping;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::extract::{extract_program_with_scoping, WaitScopes};
use super::extract_prefilter::source_may_have_queue_facts;

mod waits;

pub(crate) static HOOKS: QueueAnalysisHooks = QueueAnalysisHooks {
    semantic_demand: waits::semantic_demand,
    collect,
    scope_seeds,
};

fn collect(
    program: &Program<'_>,
    path: &Path,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
    (record_waits, scoping): (bool, Option<&Scoping>),
) -> QueueFileFacts {
    let queue_usage = plan
        .queue_usage
        .then(|| extract_queue_usage_from_program(program, source));
    let (queue_create_line, queue_name) = queue_factory_facts(program, path, source, plan, context);
    let queue_project = plan.queue_project.then(|| {
        extract_program_with_scoping(
            path,
            source,
            program,
            &context.queue_project_factory_names,
            &context.queue_factory_roles,
            record_waits,
            scoping,
        )
    });
    QueueFileFacts {
        usage: queue_usage,
        create_line: queue_create_line,
        name: queue_name,
        project: queue_project,
    }
}

fn queue_factory_facts<'a>(
    program: &Program<'a>,
    path: &Path,
    source: &str,
    plan: TsFactPlan,
    context: &TsFactContext,
) -> (Option<u32>, Option<String>) {
    if !plan.queue_factory || !context.matches_queue_factory(path) {
        return (None, None);
    }
    match (
        context.queue_factory_specifier.as_deref(),
        context.queue_factory_function.as_deref(),
    ) {
        (Some(factory_specifier), Some(factory_function)) => find_queue_factory_facts_from_program(
            program,
            source,
            factory_specifier,
            factory_function,
        ),
        _ => (None, None),
    }
}

fn scope_seeds(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    plan: &CheckFactPlan,
) -> BTreeSet<PathBuf> {
    let mut seeds = BTreeSet::new();
    let waits = std::cell::OnceCell::new();
    for path in files {
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
        ) {
            continue;
        }
        let Ok(source) = sources.read_source_path(path) else {
            seeds.insert(normalize_path(path));
            continue;
        };
        if source_may_have_queue_facts(
            &source,
            &plan.queue_factory_names,
            &plan.queue_factory_roles,
        ) || ((!plan
            .queue_factory_roles
            .enqueue_and_wait_patterns
            .is_empty()
            || source.contains('\\'))
            && waits
                .get_or_init(|| WaitScopes::new(&plan.queue_factory_roles.wait_scopes))
                .matches(path, root))
        {
            seeds.insert(normalize_path(path));
        }
    }
    seeds
}
