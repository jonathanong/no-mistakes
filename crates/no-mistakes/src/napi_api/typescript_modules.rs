#[cfg(not(coverage))]
use super::{AsyncTask, JsonValueTask};
use crate::codebase::ts_source::module_facts::{
    analyze_typescript_modules, TypeScriptModulesOptions,
};
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

pub(crate) fn analyze_typescript_modules_json_impl(
    value: serde_json::Value,
) -> napi::Result<String> {
    let options = super::options::parse_options_value::<TypeScriptModulesOptions>(value)?;
    let report = analyze_typescript_modules(&options).map_err(super::async_task::to_napi_error)?;
    Ok(serde_json::to_string(&report).expect("serializable TypeScript module facts"))
}
json_binding!(
    analyze_typescript_modules_json,
    "analyzeTypeScriptModulesJson",
    analyze_typescript_modules_json_impl
);

#[cfg(test)]
mod tests;
