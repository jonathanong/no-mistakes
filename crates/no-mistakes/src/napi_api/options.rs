use std::path::PathBuf;

use anyhow::{bail, Result as AnyhowResult};
use serde::Deserialize;
use serde_json::Value;

use crate::codebase::dependencies::RelationshipArg;
use crate::codebase::symbols::{ExportKindArg, Include, SymbolsMode};

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
    /// `react usages` target component (`path` or `path#Symbol`).
    pub(crate) target: Option<String>,
    /// `react usages` `--include` spec (comma-separated `stories,tests,props`).
    pub(crate) include: Option<String>,
}

include!("options_query.rs");

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FetchesOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) targets: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TestsPlanOptions {
    pub(crate) framework: Option<String>,
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) base: Option<String>,
    pub(crate) head: Option<String>,
    pub(crate) changed_files: Vec<String>,
    pub(crate) changed_files_file: Option<String>,
    pub(crate) diff: Option<String>,
    pub(crate) entrypoints: Vec<EntrypointOption>,
    pub(crate) include_symbols: bool,
    pub(crate) environment: Option<String>,
    pub(crate) limit_percent: Option<f64>,
    pub(crate) limit_files: Option<usize>,
    pub(crate) global_config_fallback: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TestsImpactOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) entrypoints: Vec<EntrypointOption>,
    pub(crate) include_symbols: bool,
}

include!("options_ci.rs");

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TestsWhyOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) test: Option<String>,
    pub(crate) changed: Option<String>,
    pub(crate) plan: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TestsPlanDocumentOptions {
    pub(crate) plan: Option<String>,
    pub(crate) plan_json: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PlaywrightOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) playwright_config: Vec<String>,
    pub(crate) project: Option<String>,
    pub(crate) files: Vec<String>,
    pub(crate) assert_conditional_tests: bool,
    pub(crate) allow_skipped_tests: bool,
    pub(crate) assert_unique_test_ids: bool,
    pub(crate) assert_unique_html_ids: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TraverseOptions {
    pub(crate) files: Vec<EntrypointOption>,
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) depth: Option<usize>,
    pub(crate) filters: Vec<String>,
    pub(crate) target_modules: Vec<String>,
    pub(crate) tests: Vec<String>,
    pub(crate) relationships: Vec<String>,
    pub(crate) include_symbols: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SymbolOptions {
    pub(crate) files: Vec<String>,
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) mode: Option<String>,
    pub(crate) symbol: Option<String>,
    pub(crate) kinds: Vec<String>,
    pub(crate) include: Option<String>,
}

include!("options_entrypoint.rs");

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
        "swift" => Ok(RelationshipArg::Swift),
        "terraform" => Ok(RelationshipArg::Terraform),
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

include!("options_symbols.rs");

include!("options_routing.rs");
