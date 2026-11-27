use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_source::facts::{queue_facts, TsFactContext, TsFactPlan};
use crate::codebase::ts_source::SourceStore;
use oxc_ast::ast::Program;
use oxc_semantic::Scoping;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Default)]
pub(crate) struct QueueFileFacts {
    pub usage: Option<queue_facts::usage::QueueUsage>,
    pub create_line: Option<u32>,
    pub name: Option<String>,
    pub project: Option<queue_facts::FileFacts>,
}

type SemanticDemand = fn(&Path, &TsFactContext, &str, bool) -> (bool, bool);

type Collect = fn(
    &Program<'_>,
    &Path,
    &str,
    TsFactPlan,
    &TsFactContext,
    (bool, Option<&Scoping>),
) -> QueueFileFacts;

type ScopeSeeds = fn(&Path, &[PathBuf], &SourceStore, &CheckFactPlan) -> BTreeSet<PathBuf>;

pub(crate) struct QueueAnalysisHooks {
    pub semantic_demand: SemanticDemand,
    pub collect: Collect,
    pub scope_seeds: ScopeSeeds,
}

static HOOKS: OnceLock<&'static QueueAnalysisHooks> = OnceLock::new();

pub(crate) fn register(hooks: &'static QueueAnalysisHooks) {
    HOOKS.get_or_init(|| hooks);
}

pub(crate) fn hooks() -> &'static QueueAnalysisHooks {
    HOOKS
        .get()
        .expect("queue analysis hooks are registered by the facade")
}
