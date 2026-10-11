pub(crate) fn tests_why_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<TestsWhyOptions>(options)?;
    let args = build_why_args(options).map_err(to_napi_error)?;
    to_pretty_json(&crate::tests::why::why_steps(&args).map_err(to_napi_error)?)
}

pub(crate) fn tests_comment_markdown_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<TestsPlanDocumentOptions>(options)?;
    let plan = load_plan_document(options).map_err(to_napi_error)?;
    Ok(crate::tests::comment::render_markdown_plan(&plan))
}

pub(crate) fn tests_graph_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<TestsPlanDocumentOptions>(options)?;
    let plan = load_plan_document(options).map_err(to_napi_error)?;
    to_pretty_json(&crate::tests::graph::graph_json(&plan).map_err(to_napi_error)?)
}

pub(crate) fn tests_graph_mermaid_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<TestsPlanDocumentOptions>(options)?;
    crate::tests::graph::graph_mermaid(&load_plan_document(options).map_err(to_napi_error)?)
        .map_err(to_napi_error)
}

fn load_plan_document(options: TestsPlanDocumentOptions) -> AnyhowResult<crate::tests::TestPlan> {
    match (options.plan_json, options.plan) {
        (Some(serde_json::Value::String(raw)), _) => Ok(serde_json::from_str(&raw)?),
        (Some(value), _) => Ok(serde_json::from_value(value)?),
        (None, Some(path)) => Ok(serde_json::from_str(&std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read plan from {path}"))?)?),
        (None, None) => bail!("plan or planJson is required"),
    }
}


pub(crate) fn tests_audit_json_impl(options: serde_json::Value) -> napi::Result<String> {
    let options = parse_options_value::<TestsAuditOptions>(options)?;
    let plan = load_audit_artifact(options.plan_json, options.plan, "plan").map_err(to_napi_error)?;
    let observations = load_audit_artifact(options.observations_json, options.observations, "observations").map_err(to_napi_error)?;
    let report = crate::audit_test_selection(&plan, &observations).map_err(to_napi_error)?;
    to_pretty_json(&report)
}

fn load_audit_artifact<T: serde::de::DeserializeOwned>(json: Option<serde_json::Value>, path: Option<String>, field: &str) -> AnyhowResult<T> {
    let artifact = if json.is_some() { format!("{field}Json") } else { format!("{field} file") };
    let value = match (json, path) {
        (Some(serde_json::Value::String(raw)), None) => serde_json::from_str(&raw)
            .with_context(|| format!("Invalid {field}Json JSON text; regenerate a schema_version 1 artifact"))?,
        (Some(value), None) => value,
        (None, Some(path)) => crate::tests::audit::read_artifact(Path::new(&path))?,
        _ => bail!("Exactly one of {field} and {field}Json is required"),
    };
    let value = decamelize_audit_artifact(value)
        .with_context(|| format!("Invalid {artifact} fields; regenerate a schema_version 1 artifact"))?;
    serde_json::from_value(value)
        .with_context(|| format!("Invalid {artifact} artifact shape; regenerate a schema_version 1 artifact"))
}

include!("cli_parity_audit_artifacts.rs");
