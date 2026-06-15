use serde::Serialize;

/// One resource/module/output that references a queried address.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceRefRow {
    /// The referencing block's address (e.g. `aws_lb.web`).
    pub address: String,
    /// The file containing the reference, relative to the repo root.
    pub file: String,
}

/// One output exported by a module.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleOutput {
    pub name: String,
    /// Addresses the output's `value` expression references.
    pub references: Vec<String>,
}

/// One place a module's output is consumed by a root/parent module.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputConsumer {
    pub output: String,
    /// The consuming block's address.
    pub from: String,
    /// The consuming file, relative to the repo root.
    pub file: String,
}

/// Result of `infra outputs <module-dir>`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleOutputsResult {
    /// The queried module directory, relative to the repo root.
    pub module: String,
    pub exports: Vec<ModuleOutput>,
    pub consumers: Vec<OutputConsumer>,
}

/// One test file covering a queried `.tf` file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestForRow {
    pub test_file: String,
}
