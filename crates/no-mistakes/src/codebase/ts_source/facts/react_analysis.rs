use super::TsFactContext;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use crate::fx::PathSet;
use oxc_ast::ast::Program;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

pub(crate) mod component;
pub(crate) mod server;
pub(crate) mod usages;

type ComponentExtractor =
    fn(&Path, &str, &Program<'_>, &TsFactContext) -> Arc<Vec<component::ComponentFacts>>;
type UsageExtractor = fn(&Path, &str, &Program<'_>, Option<&PathSet>) -> usages::UsageFileFacts;
type ServerRouteExtractor = fn(&Path, &str, &Program<'_>) -> server::FileFacts;
type ServerRouteProjector = fn(
    &Path,
    &TsConfig,
    Option<&TsConfigCatalog>,
    &[PathBuf],
    &dyn TsFactLookup,
    &AnalysisSession,
) -> Vec<(PathBuf, String)>;

pub(crate) struct Hooks {
    pub(crate) extract_components: ComponentExtractor,
    pub(crate) extract_usages: UsageExtractor,
    pub(crate) extract_server_routes: ServerRouteExtractor,
    pub(crate) project_server_routes: ServerRouteProjector,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub(crate) fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub(crate) fn hooks() -> &'static Hooks {
    HOOKS
        .get()
        .copied()
        .expect("React analysis hooks are registered when preparing the fact context")
}
