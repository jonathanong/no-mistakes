impl AnalyzeProjectContext {
    pub(super) fn command_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Box<RawValue>> {
        let scope = self.scope(request, options)?;
        run_command_report(request, &scope.options)
    }
}

fn run_command_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> Result<Box<RawValue>> {
    let json = super::options::command_options(request, options)?;
    match request.report_type.as_str() {
        "importers" => napi_raw(crate::napi_api::queries::importers_json_impl(json)),
        "exportsOf" => napi_raw(crate::napi_api::queries::exports_of_json_impl(json)),
        "deadExports" => napi_raw(crate::napi_api::queries::dead_exports_json_impl(json)),
        "callSites" => napi_raw(crate::napi_api::queries::call_sites_json_impl(json)),
        "resolveCheck" => napi_raw(crate::napi_api::queries::resolve_check_json_impl(json)),
        "fetches" => napi_raw(crate::napi_api::fetches_json_impl(json)),
        "dataPw" => napi_raw(crate::napi_api::data_pw_json_impl(json)),
        "registryExtension" => napi_raw(crate::napi_api::registry_extension_json_impl(json)),
        "testsPlan" => napi_raw(crate::napi_api::tests_plan_json_impl(json)),
        "testsImpact" => napi_raw(crate::napi_api::tests_impact_json_impl(json)),
        "testsTargets" => napi_raw(crate::napi_api::tests_targets_json_impl(json)),
        "testsWhy" => napi_raw(crate::napi_api::tests_why_json_impl(json)),
        "testsComment" => napi_string(crate::napi_api::tests_comment_markdown_impl(json)),
        "testsGraph" => napi_raw(crate::napi_api::tests_graph_json_impl(json)),
        "testsGraphMermaid" => napi_string(crate::napi_api::tests_graph_mermaid_impl(json)),
        "lockfileDiff" => napi_raw(crate::napi_api::lockfile_diff_json_impl(json)),
        "ciImpact" => napi_raw(crate::napi_api::ci_impact_json_impl(json)),
        "ciEnv" => napi_raw(crate::napi_api::ci_env_json_impl(json)),
        "ciTopology" => napi_raw(crate::napi_api::ci_topology_json_impl(json)),
        "impactedChecks" => napi_raw(crate::napi_api::impacted_checks_json_impl(json)),
        "infraResourceRefs" => {
            napi_raw(crate::napi_api::infra_swift::infra_resource_refs_json_impl(json))
        }
        "infraOutputs" => napi_raw(crate::napi_api::infra_swift::infra_outputs_json_impl(json)),
        "infraTestFor" => napi_raw(crate::napi_api::infra_swift::infra_test_for_json_impl(json)),
        "swiftImporters" => napi_raw(crate::napi_api::infra_swift::swift_importers_json_impl(json)),
        "swiftTestTargets" => {
            napi_raw(crate::napi_api::infra_swift::swift_test_targets_json_impl(json))
        }
        "validateMermaidMarkdown" => mermaid_report(json),
        other => bail!("unknown analyzeProject report type: {other}"),
    }
}

fn mermaid_report(json: serde_json::Value) -> Result<Box<RawValue>> {
    #[cfg(feature = "mermaid-validation")]
    {
        napi_raw(crate::napi_api::validate_mermaid_markdown_json_impl(json))
    }
    #[cfg(not(feature = "mermaid-validation"))]
    {
        let _ = json;
        bail!("validateMermaidMarkdown requires the mermaid-validation feature")
    }
}

fn napi_raw(result: napi::Result<String>) -> Result<Box<RawValue>> {
    let json = result.map_err(|error| anyhow::anyhow!("{}", error.reason))?;
    Ok(RawValue::from_string(json)?)
}

fn napi_string(result: napi::Result<String>) -> Result<Box<RawValue>> {
    let value = result.map_err(|error| anyhow::anyhow!("{}", error.reason))?;
    Ok(RawValue::from_string(serde_json::to_string(&value)?)?)
}
