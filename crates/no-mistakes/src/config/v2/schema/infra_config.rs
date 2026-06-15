use serde::{Deserialize, Serialize};

/// Top-level `infra` configuration block. Holds non-JS infrastructure analysis
/// settings. Currently only Terraform/OpenTofu.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct InfraConfig {
    pub terraform: TerraformConfig,
}

/// Terraform/OpenTofu analysis configuration.
///
/// No Terraform analysis happens unless `moduleRoots` is non-empty — there are
/// no default-on global conventions (per the "explicit opt-in" design rule).
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TerraformConfig {
    /// Directories (relative to the repo root) that contain Terraform/OpenTofu
    /// modules — both root modules and reusable child modules. Each directory is
    /// treated as one module; its `.tf` files are grouped together.
    pub module_roots: Vec<String>,
    /// File extensions to treat as Terraform sources. Defaults to `["tf"]` when
    /// empty. `.tf.json` is not parsed (HCL native syntax only).
    pub extensions: Vec<String>,
    /// How `infra test-for` maps a `.tf` file to its covering test files.
    pub test: TerraformTestConvention,
}

impl TerraformConfig {
    /// Effective source extensions, defaulting to `["tf"]`.
    pub fn effective_extensions(&self) -> Vec<String> {
        if self.extensions.is_empty() {
            vec!["tf".to_string()]
        } else {
            self.extensions.clone()
        }
    }
}

/// Configuration for the `infra test-for` test-file convention. The convention is
/// always supplied here — no test directory or suffix is hardcoded.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TerraformTestConvention {
    /// Globs locating a module's test files, anchored at the module directory by
    /// default (e.g. `["__tests__/*.test.mts"]`).
    pub test_globs: Vec<String>,
    /// When set, anchor `testGlobs` at this repo-root-relative directory instead
    /// of the module directory.
    pub test_root: Option<String>,
    /// `"resource"` (default) keeps only tests whose contents reference an address
    /// declared in the `.tf` file; `"module"` returns every test in the module.
    #[serde(rename = "match")]
    pub match_mode: Option<String>,
}
