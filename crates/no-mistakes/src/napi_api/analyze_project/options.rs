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
    let value = merged_options(request, options, true, false, true)?;
    let _: SymbolOptions = serde_json::from_value(value.clone())?;
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn project_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<String> {
    let value = merged_options(request, options, true, true, true)?;
    let project_options: ProjectOptions = serde_json::from_value(value.clone())?;
    let _ = project_roots(&project_options);
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn playwright_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<String> {
    let value = merged_options(request, options, false, false, true)?;
    let _: PlaywrightOptions = serde_json::from_value(value.clone())?;
    Ok(serde_json::to_string(&value)?)
}

pub(super) fn traverse_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<TraverseOptions> {
    let value = merged_options(request, options, true, true, false)?;
    Ok(serde_json::from_value(value)?)
}

fn merged_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    include_tsconfig: bool,
    include_filters: bool,
    include_config: bool,
) -> AnyhowResult<Value> {
    let mut map = request.options.clone();
    if let Some(root) = &options.root {
        map.entry("root".to_string())
            .or_insert_with(|| Value::String(root.clone()));
    }
    if include_tsconfig {
        if let Some(tsconfig) = &options.tsconfig {
            if !map.contains_key("tsconfig") {
                map.insert(
                    "tsconfig".to_string(),
                    Value::String(forwarded_tsconfig(options, tsconfig)?),
                );
            }
        }
    }
    if include_config {
        if let Some(config) = &options.config {
            if !map.contains_key("config") {
                map.insert(
                    "config".to_string(),
                    Value::String(forwarded_config(options, config)?),
                );
            }
        }
    }
    if include_filters && !options.filters.is_empty() {
        map.entry("filters".to_string())
            .or_insert_with(|| json!(options.filters));
    }
    Ok(Value::Object(map))
}

fn forwarded_tsconfig(options: &AnalyzeProjectOptions, tsconfig: &str) -> AnyhowResult<String> {
    forwarded_path(options, tsconfig)
}

fn forwarded_config(options: &AnalyzeProjectOptions, config: &str) -> AnyhowResult<String> {
    let path = PathBuf::from(config);
    if path.is_absolute() {
        return Ok(path.display().to_string());
    }
    Ok(resolve_forward_root(options.root.as_deref())?
        .join(path)
        .display()
        .to_string())
}

fn forwarded_path(options: &AnalyzeProjectOptions, value: &str) -> AnyhowResult<String> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        return Ok(path.display().to_string());
    }
    let Some(root) = &options.root else {
        return Ok(value.to_string());
    };
    Ok(resolve_forward_root(Some(root))?
        .join(path)
        .display()
        .to_string())
}

fn resolve_forward_root(root: Option<&str>) -> AnyhowResult<PathBuf> {
    let root = resolve_root(root)?;
    if root.is_absolute() {
        return Ok(root);
    }
    Ok(crate::codebase::ts_resolver::normalize_path(
        &std::env::current_dir()
            .context("reading current directory")?
            .join(root),
    ))
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
