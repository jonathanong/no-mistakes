// Options for the `ci` and `impacted-checks` commands. Included by options.rs.

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CiImpactOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    /// Changed file paths (relative to root or absolute).
    pub(crate) files: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CiEnvOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    /// Environment variable name (case-sensitive).
    pub(crate) var: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CiTopologyOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    /// Workflow filter(s) — basename or a path inside `.github/workflows`.
    /// Empty selects every workflow.
    pub(crate) workflows: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ImpactedChecksOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) base: Option<String>,
    pub(crate) head: Option<String>,
    pub(crate) changed_files: Vec<String>,
    pub(crate) changed_files_file: Option<String>,
    pub(crate) diff: Option<String>,
    pub(crate) timings: bool,
}
