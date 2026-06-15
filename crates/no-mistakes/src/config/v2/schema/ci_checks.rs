use serde::{Deserialize, Serialize};

/// Configuration for the `no-mistakes ci` workflow-graph commands.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CiConfig {
    /// Directories (relative to root) to scan for GitHub Actions workflow YAML.
    /// Defaults to `.github/workflows`.
    pub workflow_dirs: Vec<String>,
    /// Directories (relative to root) holding local composite actions. Recorded
    /// for future use; their internal env/permissions are not yet inlined.
    pub action_dirs: Vec<String>,
}

impl Default for CiConfig {
    fn default() -> Self {
        Self {
            workflow_dirs: vec![".github/workflows".to_string()],
            action_dirs: Vec::new(),
        }
    }
}

/// Configuration for the `no-mistakes impacted-checks` generic validation commands.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ChecksConfig {
    /// Glob-keyed validation commands (lint, typecheck, etc.).
    pub commands: Vec<CheckCommandDef>,
}

/// A single configured validation command mapped to file globs.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckCommandDef {
    /// Stable identifier used for dedupe and reporting (e.g. `eslint`, `tsc`).
    pub name: String,
    /// File globs (relative to root) that trigger this command.
    pub include: Vec<String>,
    /// File globs (relative to root) that suppress this command.
    pub exclude: Vec<String>,
    /// Command tokens, e.g. `["pnpm", "exec", "eslint"]`.
    pub command: Vec<String>,
    /// How matched file paths are added to the command invocation.
    pub file_args: CheckFileArgs,
}

impl Default for CheckCommandDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            command: Vec::new(),
            file_args: CheckFileArgs::Append,
        }
    }
}

/// How a [`CheckCommandDef`] incorporates the matched changed files.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CheckFileArgs {
    /// Append each matched file to the command as a trailing argument.
    #[default]
    Append,
    /// Run the command once regardless of which files matched (whole-project).
    None,
}
