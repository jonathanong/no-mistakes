use super::RULE_ID;
use crate::config::v2::schema::StringOrList;
use anyhow::{bail, Result};
use serde::Deserialize;

mod arguments;
pub(super) use arguments::{ArgumentConditions, ArgumentPredicate, Literal, UnknownArguments};
mod root;
pub(super) use root::{catalog_names, PlaywrightRoot, Root};

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    pub(super) roots: Vec<Root>,
    pub(super) exclude_roots: Option<StringOrList>,
    pub(super) traversal: Traversal,
    pub(super) max_depth: Option<usize>,
    pub(super) unknown_calls: UnknownCalls,
    pub(super) unknown_arguments: UnknownArguments,
    pub(super) targets: Vec<Target>,
    pub(super) invocations: Vec<Invocation>,
}
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Traversal {
    #[default]
    Direct,
    File,
    Transitive,
}
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum UnknownCalls {
    #[default]
    Finding,
    Ignore,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Invocation {
    Call,
    Construct,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Target {
    Global(GlobalTarget),
    Exact(ExactTarget),
    Terminal(TerminalTarget),
    ModuleExport(ModuleExportTarget),
    Function(FunctionTarget),
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GlobalTarget {
    pub(super) global: String,
    #[serde(default, rename = "whenArguments")]
    pub(super) when_arguments: Option<ArgumentConditions>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExactTarget {
    pub(super) exact: String,
    #[serde(default, rename = "whenArguments")]
    pub(super) when_arguments: Option<ArgumentConditions>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TerminalTarget {
    pub(super) terminal: String,
    #[serde(default, rename = "whenArguments")]
    pub(super) when_arguments: Option<ArgumentConditions>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ModuleExportTarget {
    pub(super) module_export: ModuleExportSelector,
    #[serde(default, rename = "whenArguments")]
    pub(super) when_arguments: Option<ArgumentConditions>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FunctionTarget {
    pub(super) function: FunctionSelector,
    #[serde(default, rename = "whenArguments")]
    pub(super) when_arguments: Option<ArgumentConditions>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ModuleExportSelector {
    pub(super) module: String,
    pub(super) export: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FunctionSelector {
    pub(super) file: String,
    pub(super) symbol: String,
}

pub(super) fn validate(options: &Options) -> Result<()> {
    if options.roots.is_empty() {
        bail!("{RULE_ID}: each application requires at least one options.roots selector");
    }
    if options.targets.is_empty() && options.unknown_calls != UnknownCalls::Finding {
        bail!("{RULE_ID}: each application requires targets or unknownCalls: finding");
    }
    if options.max_depth == Some(0) {
        bail!("{RULE_ID}: maxDepth must be greater than zero");
    }
    root::validate_roots(&options.roots)?;
    if let Some(exclude_roots) = &options.exclude_roots {
        let patterns = exclude_roots.values();
        if patterns.is_empty() || patterns.iter().any(|pattern| pattern.trim().is_empty()) {
            bail!("{RULE_ID}: excludeRoots must contain non-empty glob patterns");
        }
    }
    for target in &options.targets {
        if let Some(conditions) = target.arguments() {
            arguments::validate(conditions)?;
        }
        match target {
            Target::Global(GlobalTarget { global, .. })
            | Target::Exact(ExactTarget { exact: global, .. })
            | Target::Terminal(TerminalTarget {
                terminal: global, ..
            }) if global.is_empty() => {
                bail!("{RULE_ID}: call target names must not be empty")
            }
            Target::ModuleExport(ModuleExportTarget { module_export, .. })
                if module_export.module.is_empty() || module_export.export.is_empty() =>
            {
                bail!("{RULE_ID}: moduleExport requires non-empty module and export values")
            }
            Target::Function(FunctionTarget { function, .. })
                if function.file.is_empty() || function.symbol.is_empty() =>
            {
                bail!("{RULE_ID}: function requires non-empty file and symbol values")
            }
            _ => {}
        }
    }
    Ok(())
}

impl Target {
    pub(super) fn arguments(&self) -> Option<&ArgumentConditions> {
        match self {
            Self::Global(target) => target.when_arguments.as_ref(),
            Self::Exact(target) => target.when_arguments.as_ref(),
            Self::Terminal(target) => target.when_arguments.as_ref(),
            Self::ModuleExport(target) => target.when_arguments.as_ref(),
            Self::Function(target) => target.when_arguments.as_ref(),
        }
    }
}
