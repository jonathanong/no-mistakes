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

json_binding!(importers_json, "importersJson", importers_json_impl);
json_binding!(exports_of_json, "exportsOfJson", exports_of_json_impl);
json_binding!(dead_exports_json, "deadExportsJson", dead_exports_json_impl);
json_binding!(call_sites_json, "callSitesJson", call_sites_json_impl);
json_binding!(
    resolve_check_json,
    "resolveCheckJson",
    resolve_check_json_impl
);

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
    file: Option<String>,
    files: Option<Vec<String>>,
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
    let (files, batch) = match (options.file, options.files) {
        (Some(file), None) => {
            require_file(&file)?;
            (vec![PathBuf::from(file)], false)
        }
        (None, Some(files)) if !files.is_empty() => {
            if files.iter().any(|file| file.trim().is_empty()) {
                return Err(napi::Error::from_reason(
                    "files must not contain an empty path",
                ));
            }
            (files.into_iter().map(PathBuf::from).collect(), true)
        }
        _ => {
            return Err(napi::Error::from_reason(
                "exactly one of file or files is required",
            ))
        }
    };
    let args = ResolveCheckArgs {
        files,
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        format: Some(Format::Json),
        json: true,
    };
    if batch {
        crate::codebase::queries::resolve_check::run_json_batch(args)
    } else {
        crate::codebase::queries::resolve_check::run_json(args)
    }
    .map_err(to_napi_error)
}
