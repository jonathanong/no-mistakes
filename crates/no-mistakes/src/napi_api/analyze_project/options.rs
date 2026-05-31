use std::path::PathBuf;

use anyhow::{Context, Result as AnyhowResult};
use serde_json::{json, Value};

use super::types::{AnalyzeProjectOptions, AnalyzeReportRequest};
use crate::napi_api::options::{
    project_roots, PlaywrightOptions, ProjectOptions, SymbolOptions, TraverseOptions,
};

pub(super) fn symbols_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<String> {
    let value = merged_options(request, options, true, false, false);
    let _: SymbolOptions = serde_json::from_value(value.clone())?;
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn project_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<String> {
    let value = merged_options(request, options, true, true, true);
    let project_options: ProjectOptions = serde_json::from_value(value.clone())?;
    let _ = project_roots(&project_options);
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn playwright_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<String> {
    let value = merged_options(request, options, false, false, true);
    let _: PlaywrightOptions = serde_json::from_value(value.clone())?;
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn traverse_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<TraverseOptions> {
    let value = merged_options(request, options, true, true, false);
    Ok(serde_json::from_value(value)?)
}

fn merged_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    include_tsconfig: bool,
    include_filters: bool,
    include_config: bool,
) -> Value {
    let mut map = request.options.clone();
    if let Some(root) = &options.root {
        map.entry("root".to_string())
            .or_insert_with(|| Value::String(root.clone()));
    }
    if include_tsconfig {
        if let Some(tsconfig) = &options.tsconfig {
            map.entry("tsconfig".to_string())
                .or_insert_with(|| Value::String(tsconfig.clone()));
        }
    }
    if include_config {
        if let Some(config) = &options.config {
            map.entry("config".to_string())
                .or_insert_with(|| Value::String(config.clone()));
        }
    }
    if include_filters && !options.filters.is_empty() {
        map.entry("filters".to_string())
            .or_insert_with(|| json!(options.filters));
    }
    Value::Object(map)
}

pub(super) fn resolve_root(root: Option<&str>) -> AnyhowResult<PathBuf> {
    let root = match root {
        Some(root) => PathBuf::from(root),
        None => std::env::current_dir().context("reading current directory")?,
    };
    Ok(crate::codebase::ts_resolver::normalize_path(&root))
}

pub(super) fn resolve_tsconfig(
    root: &std::path::Path,
    explicit: Option<&str>,
) -> AnyhowResult<crate::codebase::dependencies::TsConfig> {
    match explicit {
        Some(path) => {
            let path = PathBuf::from(path);
            let path = if path.is_absolute() {
                path
            } else {
                root.join(path)
            };
            crate::codebase::ts_resolver::load_tsconfig(&path)
        }
        None => match crate::codebase::ts_resolver::find_tsconfig(root) {
            Some(path) => crate::codebase::ts_resolver::load_tsconfig(&path),
            None => Ok(crate::codebase::ts_resolver::TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}
