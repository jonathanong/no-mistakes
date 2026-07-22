use crate::codebase::ts_resolver::ImportResolver;
use crate::codebase::ts_routes::matcher;
use crate::codebase::ts_source::is_test_file;
use crate::server_routes::graph::PreparedServerAnalysis;
use crate::server_routes::model::ProjectReport;
use crate::server_routes::source::relative_string;
use crate::server_routes::types::ServerRoute;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerContractsReport {
    pub routes: Vec<RouteContract>,
    pub client_refs: Vec<ClientContractRef>,
    pub mismatches: Vec<QueryParamMismatch>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RouteContract {
    pub file: String,
    pub line: usize,
    pub method: String,
    pub route: String,
    pub query_params: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientContractRef {
    pub file: String,
    pub line: u32,
    pub route: String,
    pub query_params: Vec<String>,
    pub matched_route: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueryParamMismatch {
    pub file: String,
    pub line: u32,
    pub route: String,
    pub matched_route: String,
    pub missing_params: Vec<String>,
}

pub fn analyze_contracts(
    root: &Path,
    tsconfig_path: Option<&Path>,
    route_report: &ProjectReport,
    filters: &[String],
) -> anyhow::Result<ServerContractsReport> {
    let prepared = crate::server_routes::graph::prepare_analysis(root, tsconfig_path)?;
    analyze_contracts_with_prepared(&prepared, route_report, filters)
}

#[doc(hidden)]
pub fn analyze_contracts_with_prepared(
    prepared: &PreparedServerAnalysis,
    route_report: &ProjectReport,
    filters: &[String],
) -> anyhow::Result<ServerContractsReport> {
    let root = &prepared.root;
    let filter = build_filter(filters)?;
    let facts = crate::codebase::ts_source::facts::TsFactMap::from_iter_with_plan(
        prepared
            .source_files
            .iter()
            .filter(|path| !is_test_file(&relative_string(root, path)))
            .filter(|path| source_file_matches_filter(root, path, filter.as_ref()))
            .filter_map(|path| {
                prepared
                    .facts
                    .get(path)
                    .cloned()
                    .map(|facts| (path.clone(), facts))
            }),
        prepared.facts.plan(),
    );
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::from_files(
        visible.iter().cloned().collect(),
    );
    let resolver =
        ImportResolver::new_in_session(&prepared.tsconfig, Some(&visible), &prepared.session);

    let mut client_refs = Vec::new();
    let mut mismatches = Vec::new();
    {
        let mut collector = ClientContractCollector {
            root,
            route_report,
            client_refs: &mut client_refs,
            mismatches: &mut mismatches,
        };
        for (path, file_facts) in &facts {
            for route_ref in &file_facts.route_refs {
                collector.push(
                    path,
                    route_ref.line,
                    &route_ref.pattern,
                    route_ref.method.as_deref(),
                );
            }
            for (line, pattern) in
                crate::codebase::dependencies::graph::route_helper_ref_patterns_with_lines(
                    path,
                    file_facts,
                    &facts,
                    &resolver,
                    &graph_files,
                )
            {
                collector.push(path, line, &pattern, Some("GET"));
            }
        }
    }

    let mut routes: Vec<RouteContract> = route_report
        .routes
        .iter()
        .map(|route| RouteContract {
            file: route.file.clone(),
            line: route.line,
            method: route.method.clone(),
            route: route.route.clone(),
            query_params: route.query_params.clone(),
        })
        .collect();
    routes.sort_by(|a, b| (&a.route, &a.method, &a.file).cmp(&(&b.route, &b.method, &b.file)));
    client_refs.sort_by(|a, b| (&a.file, a.line, &a.route).cmp(&(&b.file, b.line, &b.route)));
    mismatches.sort_by(|a, b| (&a.file, a.line, &a.route).cmp(&(&b.file, b.line, &b.route)));

    Ok(ServerContractsReport {
        routes,
        client_refs,
        mismatches,
    })
}

include!("contracts_helpers.rs");

include!("contracts_scope.rs");

#[cfg(test)]
#[path = "contracts_test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "contracts_tests.rs"]
mod contracts_tests;
