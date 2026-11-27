use super::analysis_session::AnalysisSession;
use super::check_facts::{CheckFactMap, CheckFactPlan, CheckFactVariant, CheckFileFacts};
use super::ts_resolver::TsConfigCatalog;
use super::ts_source::{FileIdMap, SourceStore, VisiblePathSnapshot};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use oxc_ast::ast::Program;
use std::any::Any;
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub(crate) mod executor_warning_scan;
pub(crate) mod identifier_plan;
pub(crate) mod options;
pub(crate) mod scans;
pub(crate) mod verdict;

pub(crate) type FilePayload = OnceLock<Box<dyn Any + Send + Sync>>;
pub(crate) type MapPayload = Option<Arc<dyn Any + Send + Sync>>;

#[derive(Clone, Copy)]
pub(crate) struct FileInput<'a, 'ast> {
    pub(crate) root: &'a Path,
    pub(crate) path: &'a Path,
    pub(crate) source: &'a str,
    pub(crate) program: &'a Program<'ast>,
    pub(crate) plan: &'a CheckFactPlan,
}

pub(crate) enum RecoveryPath {
    Single,
    Variants,
}

type CollectProgram =
    fn(&CheckFactVariant<'_>, FileInput<'_, '_>, Option<Arc<str>>) -> CheckFileFacts;
type FileSelection = fn(&Path, &Path, &CheckFactPlan) -> bool;
type Project = fn(
    &AnalysisSession,
    &Path,
    &SourceStore,
    Option<&TsConfigCatalog>,
    &mut FileIdMap<CheckFileFacts>,
) -> u32;
type Prepare = fn(&Path, &SourceStore, &CheckFactPlan, &CheckFactMap) -> MapPayload;
type CountUnproven = fn(&FileIdMap<Arc<CheckFileFacts>>) -> u32;

type ConfigurePlan = fn(&Path, &NoMistakesConfig, &mut CheckFactPlan) -> Result<()>;
type ConfigureScope =
    fn(&Path, &NoMistakesConfig, &VisiblePathSnapshot, &mut CheckFactPlan) -> Result<()>;

pub(crate) struct Hooks {
    pub(crate) collect_program: CollectProgram,
    pub(crate) retain_source: FileSelection,
    pub(crate) recover_program: fn(FileInput<'_, '_>, RecoveryPath) -> FilePayload,
    pub(crate) failed_source: fn(&Path, &Path, &CheckFactPlan) -> FilePayload,
    pub(crate) project: Project,
    pub(crate) prepare: Prepare,
    pub(crate) count_unproven: CountUnproven,
    pub(crate) needs_import_support: fn(&CheckFileFacts) -> bool,
    pub(crate) configure_profiles: ConfigurePlan,
    pub(crate) configure_plan: ConfigurePlan,
    pub(crate) configure_helper_scope: ConfigureScope,
    pub(crate) has_sql_advice: fn(&NoMistakesConfig) -> bool,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub(crate) fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub(crate) fn hooks() -> &'static Hooks {
    HOOKS
        .get()
        .expect("Postgres analysis hooks registered by the facade")
}
