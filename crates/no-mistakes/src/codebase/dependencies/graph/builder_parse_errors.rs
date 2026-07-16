fn graph_parse_errors(
    fact_plan: TsFactPlan,
    files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
) -> HashMap<PathBuf, String> {
    if fact_plan.is_empty() {
        return HashMap::new();
    }
    facts
        .map(|facts| {
            files
                .iter()
                .filter_map(|path| {
                    facts
                        .get_ts_facts(path)
                        .and_then(|file_facts| file_facts.parse_error.as_ref())
                        .map(|error| (path.clone(), error.clone()))
                })
                .collect()
        })
        .unwrap_or_default()
}
