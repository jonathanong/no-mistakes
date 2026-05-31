use std::path::PathBuf;

use anyhow::{bail, Result as AnyhowResult};

use super::options::{
    parse_export_kind, parse_include, parse_options, parse_relationship, to_napi_error,
    SymbolOptions, TraverseOptions,
};
use crate::cli::Format;
use crate::codebase::dependencies::{Direction, TraverseArgs};
use crate::codebase::symbols::SymbolsArgs;

pub(crate) fn dependencies_json_impl(options_json: String) -> napi::Result<String> {
    traverse_json(options_json, Direction::Deps)
}

pub(crate) fn dependents_json_impl(options_json: String) -> napi::Result<String> {
    traverse_json(options_json, Direction::Dependents)
}

pub(crate) fn related_json_impl(options_json: String) -> napi::Result<String> {
    traverse_json(options_json, Direction::Dependents)
}

pub(crate) fn symbols_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<SymbolOptions>(&options_json)?;
    let args = build_symbols_args(options).map_err(to_napi_error)?;
    crate::codebase::symbols::run_json(args).map_err(to_napi_error)
}

fn traverse_json(options_json: String, direction: Direction) -> napi::Result<String> {
    let options = parse_options::<TraverseOptions>(&options_json)?;
    let args = build_traverse_args(options).map_err(to_napi_error)?;
    crate::codebase::dependencies::run_json(args, direction).map_err(to_napi_error)
}

pub(crate) fn build_traverse_args(options: TraverseOptions) -> AnyhowResult<TraverseArgs> {
    if options.files.is_empty() {
        bail!("files must contain at least one file");
    }

    Ok(TraverseArgs {
        files: entrypoints_to_paths(options.files),
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        depth: options.depth,
        filters: options.filters,
        target_modules: options.target_modules,
        tests: options.tests,
        format: Some(Format::Json),
        json: true,
        relationships: options
            .relationships
            .iter()
            .map(|value| parse_relationship(value))
            .collect::<AnyhowResult<Vec<_>>>()?,
        symbols: options.include_symbols,
        timings: false,
    })
}

fn build_symbols_args(options: SymbolOptions) -> AnyhowResult<SymbolsArgs> {
    if options.files.is_empty() {
        bail!("files must contain at least one file");
    }

    Ok(SymbolsArgs {
        files: strings_to_paths(options.files),
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        kinds: options
            .kinds
            .iter()
            .map(|value| parse_export_kind(value))
            .collect::<AnyhowResult<Vec<_>>>()?,
        include: parse_include(options.include.as_deref())?,
        format: Some(Format::Json),
        json: true,
        timings: false,
    })
}

fn strings_to_paths(values: Vec<String>) -> Vec<PathBuf> {
    values.into_iter().map(PathBuf::from).collect()
}

fn entrypoints_to_paths(values: Vec<super::options::EntrypointOption>) -> Vec<PathBuf> {
    values
        .into_iter()
        .map(|value| PathBuf::from(value.into_cli_string()))
        .collect()
}
