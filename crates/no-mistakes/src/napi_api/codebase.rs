use std::path::PathBuf;

use anyhow::{bail, Result as AnyhowResult};

use super::options::{
    parse_export_kind, parse_include, parse_options, parse_relationship, parse_symbols_mode,
    to_napi_error, ImportUsagesOptions, SymbolOptions, TraverseOptions,
};
use crate::cli::Format;
use crate::codebase::dependencies::{Direction, TraverseArgs};
use crate::codebase::import_usages::ImportUsagesArgs;
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

pub(crate) fn import_usages_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ImportUsagesOptions>(&options_json)?;
    let args = build_import_usages_args(options);
    crate::codebase::import_usages::run_json(args).map_err(to_napi_error)
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
        files: entrypoint_files(&options.files),
        file_entrypoints_are_structured: entrypoint_structured(&options.files),
        file_symbols: entrypoint_symbols(options.files),
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
        include_symbols: options.include_symbols,
        timings: false,
    })
}

pub(crate) fn build_import_usages_args(options: ImportUsagesOptions) -> ImportUsagesArgs {
    ImportUsagesArgs {
        files: strings_to_paths(options.files),
        root: options.root.map(PathBuf::from),
        scan_roots: strings_to_paths(options.scan_roots),
        filters: options.filters,
        format: Some(Format::Json),
        json: true,
        timings: false,
    }
}

pub(crate) fn build_symbols_args(options: SymbolOptions) -> AnyhowResult<SymbolsArgs> {
    if options.files.is_empty() {
        bail!("files must contain at least one file");
    }

    Ok(SymbolsArgs {
        files: strings_to_paths(options.files),
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        config: options.config.map(PathBuf::from),
        mode: parse_symbols_mode(options.mode.as_deref())?,
        symbol: options.symbol,
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

fn entrypoint_files(values: &[super::options::EntrypointOption]) -> Vec<PathBuf> {
    values
        .iter()
        .cloned()
        .map(|value| PathBuf::from(value.into_parts().0))
        .collect()
}

fn entrypoint_symbols(values: Vec<super::options::EntrypointOption>) -> Vec<Option<String>> {
    values
        .into_iter()
        .map(|value| value.into_parts().1)
        .collect()
}

fn entrypoint_structured(values: &[super::options::EntrypointOption]) -> Vec<bool> {
    values.iter().map(|value| value.is_structured()).collect()
}

#[cfg(test)]
mod tests;
