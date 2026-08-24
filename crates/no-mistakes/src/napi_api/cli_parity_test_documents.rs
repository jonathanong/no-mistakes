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
