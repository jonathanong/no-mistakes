use super::*;
use crate::codebase::ts_source::facts::{
    collect_ts_facts_with_context_sources_and_session, TsFactContext, TsFactMap, TsFactPlan,
};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeScriptModulesOptions {
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub root: Option<PathBuf>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptModule {
    pub file_name: PathBuf,
    pub complete: bool,
    #[serde(flatten)]
    pub facts: TypeScriptModuleFacts,
}
#[derive(Debug, Serialize)]
pub struct TypeScriptModulesReport {
    pub modules: Vec<TypeScriptModule>,
}

/// Analyze only explicitly selected modules in one request-local source/fact pass.
pub fn analyze_typescript_modules(
    options: &TypeScriptModulesOptions,
) -> anyhow::Result<TypeScriptModulesReport> {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::env::current_dir()?
            .join(options.root.as_deref().unwrap_or(std::path::Path::new("."))),
    );
    anyhow::ensure!(
        root.is_dir(),
        "TypeScript module root must be an existing directory: {}",
        root.display()
    );
    let files = options
        .files
        .iter()
        .map(|path| {
            crate::codebase::ts_resolver::normalize_path(&if path.is_absolute() {
                path.clone()
            } else {
                root.join(path)
            })
        })
        .collect::<Vec<_>>();
    let files = crate::codebase::ts_source::deduplicate_analysis_paths(files.iter());
    let context = TsFactContext {
        root,
        ..TsFactContext::default()
    };
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let sources = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        session.observer().cloned(),
    );
    let map = collect_ts_facts_with_context_sources_and_session(
        &session,
        &files,
        TsFactPlan {
            module_bindings: true,
            ..TsFactPlan::default()
        },
        &context,
        &sources,
    );
    Ok(project_modules(files, map))
}

// The owner may union module demand with other facts and then consume the map
// here, transferring the uniquely owned payload without copying its vectors.
pub(super) fn project_modules(files: Vec<PathBuf>, map: TsFactMap) -> TypeScriptModulesReport {
    let mut map = map.into_iter().collect::<crate::fx::FxHashMap<_, _>>();
    let modules = files
        .into_iter()
        .map(|file_name| {
            let mut collected = map.remove(&file_name);
            let mut facts = collected
                .as_mut()
                .and_then(|facts| facts.module_bindings.take())
                .map(Arc::unwrap_or_clone)
                .unwrap_or_default();
            if let Some(collected) = &collected {
                if let Some(error) = &collected.operational_error {
                    facts.diagnostics.push(ModuleDiagnostic {
                        kind: "sourceError".into(),
                        message: error.clone(),
                        span: None,
                    });
                } else if let Some(error) = &collected.parse_error {
                    facts.diagnostics.push(ModuleDiagnostic {
                        kind: "parseError".into(),
                        message: error.clone(),
                        span: None,
                    });
                }
            } else {
                facts.diagnostics.push(ModuleDiagnostic {
                    kind: "sourceError".into(),
                    message: "Unsupported source extension or incomplete collection".into(),
                    span: None,
                });
            }
            TypeScriptModule {
                file_name,
                complete: facts.diagnostics.is_empty(),
                facts,
            }
        })
        .collect();
    TypeScriptModulesReport { modules }
}
