#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FlowOptions {
    pub(crate) target: Option<String>,
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) direction: Option<String>,
    pub(crate) depth: Option<usize>,
    pub(crate) relationships: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TestsTargetsOptions {
    pub(crate) framework: Option<String>,
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) files: Vec<String>,
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
    pub(crate) from_git_diff: Option<String>,
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
