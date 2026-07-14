pub(super) fn traversal_report_keys(
    options: &AnalyzeProjectOptions,
) -> Result<(std::collections::HashSet<String>, std::collections::HashSet<String>)> {
    let mut queue = std::collections::HashSet::new();
    let mut server = std::collections::HashSet::new();
    for request in &options.reports {
        if !matches!(request.report_type.as_str(),
            "queueEdges" | "queueRelated" | "serverRouteEdges" | "serverRouteRelated")
        {
            continue;
        }
        let raw = project_options(request, options)?;
        let parsed: ProjectOptions = serde_json::from_str(&raw)?;
        if matches!(request.report_type.as_str(), "queueEdges" | "queueRelated") {
            queue.insert(canonical_filter_key(&parsed.filters)?);
        } else {
            server.insert(canonical_filter_key(&server_filters(&request.report_type, &parsed))?);
        }
    }
    Ok((queue, server))
}
