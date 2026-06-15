// Included into `napi_api::options` via `include!`; shares that module's
// imports. Option structs for the issue-419 query commands.

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DataPwOptions {
    pub(crate) root: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) attributes: Vec<String>,
    pub(crate) scan: Vec<String>,
    pub(crate) include: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EffectsOptions {
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) entry: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) depth: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RscCallersOptions {
    pub(crate) root: Option<String>,
    pub(crate) tsconfig: Option<String>,
    pub(crate) config: Option<String>,
    pub(crate) component: Option<String>,
    pub(crate) depth: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RegistryExtensionOptions {
    pub(crate) root: Option<String>,
    pub(crate) registry_file: Option<String>,
}
