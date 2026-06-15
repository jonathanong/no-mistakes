use std::path::PathBuf;

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;
use serde::Deserialize;

#[cfg(not(coverage))]
use super::async_task::JsonTask;
use super::options::{parse_options, to_napi_error};
use crate::cli::Format;
use crate::codebase::queries::{
    CallSitesArgs, DeadExportsArgs, ExportsOfArgs, ImportersArgs, ResolveCheckArgs,
};

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "importersJson"))]
pub fn importers_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, importers_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "exportsOfJson"))]
pub fn exports_of_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, exports_of_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "deadExportsJson"))]
pub fn dead_exports_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, dead_exports_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "callSitesJson"))]
pub fn call_sites_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, call_sites_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "resolveCheckJson"))]
pub fn resolve_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, resolve_check_json_impl))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ImportersOptions {
    file: String,
    tests: bool,
    root: Option<String>,
    tsconfig: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ExportsOfOptions {
    file: String,
    no_importers: bool,
    root: Option<String>,
    tsconfig: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct DeadExportsOptions {
    file: String,
    names: Vec<String>,
    root: Option<String>,
    tsconfig: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct CallSitesOptions {
    file: String,
    export_name: String,
    root: Option<String>,
    tsconfig: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ResolveCheckOptions {
    file: String,
    root: Option<String>,
    tsconfig: Option<String>,
}

fn require_file(file: &str) -> napi::Result<()> {
    if file.trim().is_empty() {
        return Err(napi::Error::from_reason("file is required"));
    }
    Ok(())
}

pub(crate) fn importers_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ImportersOptions>(&options_json)?;
    require_file(&options.file)?;
    crate::codebase::queries::importers::run_json(ImportersArgs {
        file: PathBuf::from(options.file),
        tests: options.tests,
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(Format::Json),
        json: true,
    })
    .map_err(to_napi_error)
}

pub(crate) fn exports_of_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ExportsOfOptions>(&options_json)?;
    require_file(&options.file)?;
    crate::codebase::queries::exports_of::run_json(ExportsOfArgs {
        file: PathBuf::from(options.file),
        no_importers: options.no_importers,
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(Format::Json),
        json: true,
    })
    .map_err(to_napi_error)
}

pub(crate) fn dead_exports_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<DeadExportsOptions>(&options_json)?;
    require_file(&options.file)?;
    crate::codebase::queries::dead_exports::run_json(DeadExportsArgs {
        file: PathBuf::from(options.file),
        names: options.names,
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(Format::Json),
        json: true,
    })
    .map_err(to_napi_error)
}

pub(crate) fn call_sites_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<CallSitesOptions>(&options_json)?;
    require_file(&options.file)?;
    if options.export_name.trim().is_empty() {
        return Err(napi::Error::from_reason("exportName is required"));
    }
    crate::codebase::queries::call_sites::run_json(CallSitesArgs {
        file: PathBuf::from(options.file),
        export_name: options.export_name,
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(Format::Json),
        json: true,
    })
    .map_err(to_napi_error)
}

pub(crate) fn resolve_check_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ResolveCheckOptions>(&options_json)?;
    require_file(&options.file)?;
    crate::codebase::queries::resolve_check::run_json(ResolveCheckArgs {
        file: PathBuf::from(options.file),
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(Format::Json),
        json: true,
    })
    .map_err(to_napi_error)
}
