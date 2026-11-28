use super::options::{parse_options_value, to_napi_error};
use no_mistakes::changes::ChangesDiffArgs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct ChangesDiffOptions {
    base_root: Option<PathBuf>,
    head_root: Option<PathBuf>,
    config: Option<PathBuf>,
}

pub(crate) fn changes_diff_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let opts: ChangesDiffOptions = parse_options_value(options)?;
    let args = ChangesDiffArgs {
        base_root: opts
            .base_root
            .ok_or_else(|| napi::Error::from_reason("baseRoot is required"))?,
        head_root: opts
            .head_root
            .ok_or_else(|| napi::Error::from_reason("headRoot is required"))?,
        config: opts.config,
    };
    let report = no_mistakes::changes::diff(&args).map_err(to_napi_error)?;
    serde_json::to_string(&report).map_err(|error| to_napi_error(error.into()))
}

#[cfg(test)]
mod tests;
