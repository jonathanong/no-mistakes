use super::RULE_ID;
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    pub(super) roots: Vec<Root>,
    pub(super) traversal: Traversal,
    pub(super) max_depth: Option<usize>,
    pub(super) unknown_calls: UnknownCalls,
    pub(super) targets: Vec<Target>,
    pub(super) invocations: Vec<Invocation>,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Root {
    File(FileRoot),
    Module(ModuleRoot),
    Function(FunctionRoot),
    Vitest(VitestRoot),
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileRoot {
    pub(super) file: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ModuleRoot {
    pub(super) module: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FunctionRoot {
    pub(super) function: FunctionSelector,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VitestRoot {
    pub(super) vitest: VitestSelector,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum VitestSelector {
    All(bool),
    Projects(Vec<String>),
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Traversal {
    Direct,
    File,
    Transitive,
}
impl Default for Traversal {
    fn default() -> Self {
        Self::Direct
    }
}
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum UnknownCalls {
    #[default]
    Ignore,
    Finding,
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
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExactTarget {
    pub(super) exact: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TerminalTarget {
    pub(super) terminal: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ModuleExportTarget {
    pub(super) module_export: ModuleExportSelector,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FunctionTarget {
    pub(super) function: FunctionSelector,
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
    for root in &options.roots {
        match root {
            Root::Vitest(VitestRoot {
                vitest: VitestSelector::All(false),
            }) => bail!("{RULE_ID}: vitest root must be true or a non-empty project list"),
            Root::Vitest(VitestRoot {
                vitest: VitestSelector::Projects(names),
            }) if names.is_empty() => {
                bail!("{RULE_ID}: vitest root project list must not be empty")
            }
            _ => {}
        }
    }
    for target in &options.targets {
        match target {
            Target::Global(GlobalTarget { global })
            | Target::Exact(ExactTarget { exact: global })
            | Target::Terminal(TerminalTarget { terminal: global })
                if global.is_empty() =>
            {
                bail!("{RULE_ID}: call target names must not be empty")
            }
            Target::ModuleExport(ModuleExportTarget { module_export })
                if module_export.module.is_empty() || module_export.export.is_empty() =>
            {
                bail!("{RULE_ID}: moduleExport requires non-empty module and export values")
            }
            Target::Function(FunctionTarget { function })
                if function.file.is_empty() || function.symbol.is_empty() =>
            {
                bail!("{RULE_ID}: function requires non-empty file and symbol values")
            }
            _ => {}
        }
    }
    Ok(())
}
