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

pub mod executor_warning_scan;
pub mod identifier_plan;
pub mod options;
pub mod scans;
pub mod verdict;

pub type FilePayload = OnceLock<Box<dyn Any + Send + Sync>>;
pub type MapPayload = Option<Arc<dyn Any + Send + Sync>>;

#[derive(Clone, Copy)]
pub struct FileInput<'a, 'ast> {
    pub root: &'a Path,
    pub path: &'a Path,
    pub source: &'a str,
    pub program: &'a Program<'ast>,
    pub plan: &'a CheckFactPlan,
}

pub enum RecoveryPath {
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

pub struct Hooks {
    pub(crate) collect_program: CollectProgram,
    pub retain_source: FileSelection,
    pub recover_program: fn(FileInput<'_, '_>, RecoveryPath) -> FilePayload,
    pub failed_source: fn(&Path, &Path, &CheckFactPlan) -> FilePayload,
    pub project: Project,
    pub prepare: Prepare,
    pub count_unproven: CountUnproven,
    pub needs_import_support: fn(&CheckFileFacts) -> bool,
    pub configure_profiles: ConfigurePlan,
    pub configure_plan: ConfigurePlan,
    pub configure_helper_scope: ConfigureScope,
    pub has_sql_advice: fn(&NoMistakesConfig) -> bool,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub fn hooks() -> &'static Hooks {
    HOOKS
        .get()
        .expect("Postgres analysis hooks registered by the facade")
}
