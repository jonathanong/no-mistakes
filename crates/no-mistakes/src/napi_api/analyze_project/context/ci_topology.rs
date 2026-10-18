fn prepare_ci_topology(
    options: &AnalyzeProjectOptions,
    traversal: &SharedTraversalContext,
    check: Option<&SharedCheckContext>,
) -> Option<std::sync::Arc<crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs>> {
    use crate::codebase::ci_workflows::{ParsedWorkflowSet, PreparedWorkflowTopologyInputs};
    if !options
        .reports
        .iter()
        .any(|request| request.report_type == "ciTopology")
        && !traversal.build_plan().ci
        && !traversal.build_plan().workflow_topology
    {
        return None;
    }
    Some(
        check
            .and_then(|check| check.prepared.workflow_documents.clone())
            .unwrap_or_else(|| {
                let root = traversal.root();
                let config = traversal.config();
                std::sync::Arc::new(PreparedWorkflowTopologyInputs::new(
                    root,
                    &config.ci,
                    std::sync::Arc::new(ParsedWorkflowSet::load_from_snapshot_and_sources(
                        root,
                        &config.ci,
                        traversal.visible_paths(),
                        &traversal.source_store(),
                    )),
                    ParsedWorkflowSet {
                        documents: Vec::new(),
                    },
                ))
            }),
    )
}

impl PreparedScope {
    fn ci_topology_report(
        &self,
        request: &AnalyzeReportRequest,
        options: &AnalyzeProjectOptions,
    ) -> Result<Box<RawValue>> {
        let json = super::options::command_options(request, options)?;
        let options = crate::napi_api::options::parse_options_value::<
            crate::napi_api::options::CiTopologyOptions,
        >(json)
        .map_err(|error| anyhow::anyhow!("{}", error.reason))?;
        let prepared = self
            .ci_topology
            .as_ref()
            .context("prepared CI topology owner is missing")?;
        Ok(json_raw_report(
            prepared.project_topology(&options.workflows).as_ref(),
        ))
    }
}

#[cfg(test)]
#[path = "ci_topology_tests.rs"]
mod ci_topology_tests;

#[cfg(test)]
#[path = "ci_topology_demand_tests.rs"]
mod ci_topology_demand_tests;
