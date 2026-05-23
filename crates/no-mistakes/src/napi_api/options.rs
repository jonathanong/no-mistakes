use std::path::PathBuf;

use anyhow::{bail, Result as AnyhowResult};
use serde::Deserialize;

use crate::codebase::dependencies::RelationshipArg;
use crate::codebase::symbols::{ExportKindArg, Include};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ProjectOptions {
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) filters: Vec<String>,
    pub(crate) targets: Vec<String>,
    pub(crate) files: Vec<String>,
    pub(crate) roots: Vec<String>,
    pub(crate) depth: Option<usize>,
    pub(crate) assert_no_fetch: bool,
    pub(crate) direction: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TraverseOptions {
    pub(crate) files: Vec<String>,
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) depth: Option<usize>,
    pub(crate) filters: Vec<String>,
    pub(crate) target_modules: Vec<String>,
    pub(crate) tests: Vec<String>,
    pub(crate) relationships: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SymbolOptions {
    pub(crate) files: Vec<String>,
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) kinds: Vec<String>,
    pub(crate) include: Option<String>,
}

pub(crate) fn parse_options<T: for<'de> Deserialize<'de>>(options_json: &str) -> napi::Result<T> {
    serde_json::from_str(options_json)
        .map_err(|error| napi::Error::from_reason(format!("invalid options JSON: {error}")))
}

pub(crate) fn resolve_project_root(root: Option<&str>) -> AnyhowResult<PathBuf> {
    match root {
        Some(root) => Ok(PathBuf::from(root)),
        None => std::env::current_dir().map_err(Into::into),
    }
}

pub(crate) fn project_roots(options: &ProjectOptions) -> Vec<String> {
    if options.roots.is_empty() {
        options.files.clone()
    } else {
        options.roots.clone()
    }
}

pub(crate) fn parse_relationship(value: &str) -> AnyhowResult<RelationshipArg> {
    match value {
        "import" => Ok(RelationshipArg::Import),
        "import-static" => Ok(RelationshipArg::ImportStatic),
        "import-dynamic" => Ok(RelationshipArg::ImportDynamic),
        "import-type" => Ok(RelationshipArg::ImportType),
        "import-require" => Ok(RelationshipArg::ImportRequire),
        "workspace" => Ok(RelationshipArg::Workspace),
        "package" => Ok(RelationshipArg::Package),
        "test" => Ok(RelationshipArg::Test),
        "route" => Ok(RelationshipArg::Route),
        "queue" => Ok(RelationshipArg::Queue),
        "md" => Ok(RelationshipArg::Md),
        "ci" => Ok(RelationshipArg::Ci),
        "http" => Ok(RelationshipArg::Http),
        "process" => Ok(RelationshipArg::Process),
        "asset" => Ok(RelationshipArg::Asset),
        "react" => Ok(RelationshipArg::React),
        "all" => Ok(RelationshipArg::All),
        _ => bail!("unknown relationship: {value}"),
    }
}

pub(crate) fn parse_export_kind(value: &str) -> AnyhowResult<ExportKindArg> {
    match value {
        "function" => Ok(ExportKindArg::Function),
        "class" => Ok(ExportKindArg::Class),
        "const" => Ok(ExportKindArg::Const),
        "let" => Ok(ExportKindArg::Let),
        "var" => Ok(ExportKindArg::Var),
        "type" => Ok(ExportKindArg::Type),
        "interface" => Ok(ExportKindArg::Interface),
        "enum" => Ok(ExportKindArg::Enum),
        "default" => Ok(ExportKindArg::Default),
        "re-export" => Ok(ExportKindArg::ReExport),
        _ => bail!("unknown export kind: {value}"),
    }
}

pub(crate) fn parse_include(value: Option<&str>) -> AnyhowResult<Include> {
    match value.unwrap_or("exports") {
        "exports" => Ok(Include::Exports),
        "imports" => Ok(Include::Imports),
        "both" => Ok(Include::Both),
        value => bail!("unknown include value: {value}"),
    }
}

pub(crate) fn parse_queue_direction(
    value: Option<&str>,
) -> AnyhowResult<crate::queue::RelatedDirection> {
    match value.unwrap_or("both") {
        "deps" => Ok(crate::queue::RelatedDirection::Deps),
        "dependents" => Ok(crate::queue::RelatedDirection::Dependents),
        "both" => Ok(crate::queue::RelatedDirection::Both),
        value => bail!("unknown direction: {value}"),
    }
}

pub(crate) fn parse_server_direction(
    value: Option<&str>,
) -> AnyhowResult<crate::server_routes::RelatedDirection> {
    match value.unwrap_or("both") {
        "deps" => Ok(crate::server_routes::RelatedDirection::Deps),
        "dependents" => Ok(crate::server_routes::RelatedDirection::Dependents),
        "both" => Ok(crate::server_routes::RelatedDirection::Both),
        value => bail!("unknown direction: {value}"),
    }
}

pub(crate) fn to_napi_error(error: anyhow::Error) -> napi::Error {
    napi::Error::from_reason(format!("{error:#}"))
}
