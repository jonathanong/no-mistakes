use super::*;
use crate::codebase::ts_source::facts::{collect_ts_facts_with_context, TsFactContext, TsFactPlan};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    let map = collect_ts_facts_with_context(
        &files,
        TsFactPlan {
            module_bindings: true,
            ..TsFactPlan::default()
        },
        &context,
    );
    let modules = files
        .into_iter()
        .map(|file_name| {
            let mut facts = map
                .get(&file_name)
                .and_then(|facts| facts.module_bindings.as_deref().cloned())
                .unwrap_or_default();
            if let Some(collected) = map.get(&file_name) {
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
    Ok(TypeScriptModulesReport { modules })
}
