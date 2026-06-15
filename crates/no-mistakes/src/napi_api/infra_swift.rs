use std::path::PathBuf;

use serde::Deserialize;

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

use super::options::{parse_options, resolve_project_root, to_napi_error};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct InfraOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    /// `resource-refs` address (`<type>.<name>`).
    pub(crate) address: Option<String>,
    /// `outputs` module directory.
    pub(crate) module_dir: Option<String>,
    /// `test-for` `.tf` file.
    pub(crate) tf_file: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SwiftOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    /// The Swift source file to query.
    pub(crate) file: Option<String>,
}

fn infra_report(options: &InfraOptions) -> napi::Result<crate::terraform_api::InfraReport> {
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    crate::terraform_api::analyze_project(&root, config.as_deref()).map_err(to_napi_error)
}

fn to_pretty<T: serde::Serialize>(value: &T) -> napi::Result<String> {
    serde_json::to_string_pretty(value).map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn infra_resource_refs_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<InfraOptions>(&options_json)?;
    let address = options
        .address
        .clone()
        .ok_or_else(|| napi::Error::from_reason("address is required".to_string()))?;
    let report = infra_report(&options)?;
    to_pretty(&report.resource_refs(&address))
}

pub(crate) fn infra_outputs_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<InfraOptions>(&options_json)?;
    let module_dir = options
        .module_dir
        .clone()
        .ok_or_else(|| napi::Error::from_reason("moduleDir is required".to_string()))?;
    let report = infra_report(&options)?;
    to_pretty(&report.outputs(&module_dir))
}

pub(crate) fn infra_test_for_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<InfraOptions>(&options_json)?;
    let tf_file = options
        .tf_file
        .clone()
        .ok_or_else(|| napi::Error::from_reason("tfFile is required".to_string()))?;
    let report = infra_report(&options)?;
    to_pretty(&report.test_for(&tf_file))
}

fn swift_report(options: &SwiftOptions) -> napi::Result<crate::swift_api::SwiftReport> {
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    crate::swift_api::analyze_project(&root, config.as_deref()).map_err(to_napi_error)
}

pub(crate) fn swift_importers_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<SwiftOptions>(&options_json)?;
    let file = options
        .file
        .clone()
        .ok_or_else(|| napi::Error::from_reason("file is required".to_string()))?;
    let report = swift_report(&options)?;
    to_pretty(&report.importers(&file))
}

pub(crate) fn swift_test_targets_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<SwiftOptions>(&options_json)?;
    let file = options
        .file
        .clone()
        .ok_or_else(|| napi::Error::from_reason("file is required".to_string()))?;
    let report = swift_report(&options)?;
    to_pretty(&report.test_targets(&file))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "infraResourceRefsJson"))]
pub fn infra_resource_refs_json(options_json: String) -> AsyncTask<super::async_task::JsonTask> {
    AsyncTask::new(super::async_task::JsonTask::new(
        options_json,
        infra_resource_refs_json_impl,
    ))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "infraOutputsJson"))]
pub fn infra_outputs_json(options_json: String) -> AsyncTask<super::async_task::JsonTask> {
    AsyncTask::new(super::async_task::JsonTask::new(
        options_json,
        infra_outputs_json_impl,
    ))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "infraTestForJson"))]
pub fn infra_test_for_json(options_json: String) -> AsyncTask<super::async_task::JsonTask> {
    AsyncTask::new(super::async_task::JsonTask::new(
        options_json,
        infra_test_for_json_impl,
    ))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "swiftImportersJson"))]
pub fn swift_importers_json(options_json: String) -> AsyncTask<super::async_task::JsonTask> {
    AsyncTask::new(super::async_task::JsonTask::new(
        options_json,
        swift_importers_json_impl,
    ))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "swiftTestTargetsJson"))]
pub fn swift_test_targets_json(options_json: String) -> AsyncTask<super::async_task::JsonTask> {
    AsyncTask::new(super::async_task::JsonTask::new(
        options_json,
        swift_test_targets_json_impl,
    ))
}

#[cfg(test)]
mod tests;
