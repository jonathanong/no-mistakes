// Included into `napi_api::project` via `include!`; shares that module's
// imports. JSON impls for the issue-419 query commands.

pub(crate) fn data_pw_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<DataPwOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let config = options.config.as_deref().map(PathBuf::from);
    let value = options
        .value
        .ok_or_else(|| napi::Error::from_reason("value is required for data-pw".to_string()))?;
    let include = crate::data_pw_query::DataPwInclude::parse(options.include.as_deref())
        .map_err(to_napi_error)?;
    let report = crate::data_pw_query::run(
        &root,
        config.as_deref(),
        &value,
        &options.attributes,
        &options.scan,
        &include,
    )
    .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn effects_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<EffectsOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let config = options.config.as_deref().map(PathBuf::from);
    let kind = options
        .kind
        .ok_or_else(|| napi::Error::from_reason("kind is required for effects".to_string()))?;
    let entry = options
        .entry
        .ok_or_else(|| napi::Error::from_reason("entry is required for effects".to_string()))?;
    let report = crate::effects_query::run(
        &root,
        config.as_deref(),
        tsconfig.as_deref(),
        &kind,
        &PathBuf::from(entry),
        &options.categories,
        options.depth,
    )
    .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn rsc_callers_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<RscCallersOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let tsconfig = options.tsconfig.as_deref().map(PathBuf::from);
    let config = options.config.as_deref().map(PathBuf::from);
    let component = options.component.ok_or_else(|| {
        napi::Error::from_reason("component is required for rsc-callers".to_string())
    })?;
    let report = crate::rsc_callers_query::run(
        &root,
        config.as_deref(),
        tsconfig.as_deref(),
        &PathBuf::from(component),
        options.depth,
    )
    .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

pub(crate) fn registry_extension_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<RegistryExtensionOptions>(&options_json)?;
    let root = resolve_project_root(options.root.as_deref()).map_err(to_napi_error)?;
    let registry_file = options.registry_file.ok_or_else(|| {
        napi::Error::from_reason("registryFile is required for registry-extension".to_string())
    })?;
    let report = crate::registry_extension_query::run(&root, &PathBuf::from(registry_file))
        .map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}
