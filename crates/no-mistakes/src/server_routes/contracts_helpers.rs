struct ClientContractCollector<'a> {
    root: &'a Path,
    route_report: &'a ProjectReport,
    client_refs: &'a mut Vec<ClientContractRef>,
    mismatches: &'a mut Vec<QueryParamMismatch>,
}

impl ClientContractCollector<'_> {
    fn push(&mut self, path: &Path, line: u32, pattern: &str, method: Option<&str>) {
        let Some(query_params) = query_params_from_pattern(pattern) else {
            return;
        };
        let route_path = path_without_query(pattern);
        let matched = matching_route(self.route_report, &route_path, method);
        if let Some(route) = matched {
            let missing = missing_query_params(&query_params, &route.query_params);
            if !missing.is_empty() {
                self.mismatches.push(QueryParamMismatch {
                    file: relative_string(self.root, path),
                    line,
                    route: route_path.clone(),
                    matched_route: route.route.clone(),
                    missing_params: missing,
                });
            }
        }
        self.client_refs.push(ClientContractRef {
            file: relative_string(self.root, path),
            line,
            route: route_path,
            query_params,
            matched_route: matched.map(|route| route.route.clone()),
        });
    }
}

fn matching_route<'a>(
    report: &'a ProjectReport,
    route_path: &str,
    method: Option<&str>,
) -> Option<&'a ServerRoute> {
    report.routes.iter().find(|route| {
        method.is_none_or(|method| route.method.eq_ignore_ascii_case(method))
            && matcher::matches(route_path, &route.route)
    })
}

fn missing_query_params(client: &[String], server: &[String]) -> Vec<String> {
    let server: BTreeSet<&str> = server.iter().map(String::as_str).collect();
    client
        .iter()
        .filter(|param| !server.contains(param.as_str()))
        .cloned()
        .collect()
}

fn query_params_from_pattern(pattern: &str) -> Option<Vec<String>> {
    let query = pattern.split_once('?')?.1.split('#').next().unwrap_or("");
    let mut params: BTreeSet<String> = BTreeSet::new();
    for pair in query.split('&') {
        let name = pair.split_once('=').map_or(pair, |(name, _)| name);
        if !name.is_empty() && !name.starts_with(':') {
            params.insert(name.to_string());
        }
    }
    (!params.is_empty()).then(|| params.into_iter().collect())
}

fn path_without_query(pattern: &str) -> String {
    pattern
        .split('?')
        .next()
        .unwrap_or(pattern)
        .split('#')
        .next()
        .unwrap_or(pattern)
        .to_string()
}
