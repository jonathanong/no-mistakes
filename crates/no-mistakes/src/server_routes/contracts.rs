use crate::codebase::ts_resolver::{find_tsconfig, load_tsconfig, ImportResolver, TsConfig};
use crate::codebase::ts_routes::matcher;
use crate::codebase::ts_source::is_test_file;
use crate::config::v2::load_v2_config;
use crate::server_routes::model::ProjectReport;
use crate::server_routes::source::{discover_source_files, relative_string};
use crate::server_routes::types::ServerRoute;
use anyhow::Context;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

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
    let root = root.canonicalize().unwrap_or(root.to_path_buf());
    let tsconfig = resolve_tsconfig(&root, tsconfig_path)?;
    let config = load_v2_config(&root, None).ok();
    let extra_skip = config
        .as_ref()
        .map(|config| config.filesystem.skip_directories.as_slice())
        .unwrap_or(&[]);
    let filter = build_filter(filters)?;
    let files = contract_source_files(&root, extra_skip, filter.as_ref());
    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        &files,
        crate::codebase::ts_source::facts::TsFactPlan {
            route_refs: true,
            ..Default::default()
        },
        &crate::codebase::ts_source::facts::TsFactContext::new(&root),
    );
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);

    let mut client_refs = Vec::new();
    let mut mismatches = Vec::new();
    {
        let mut collector = ClientContractCollector {
            root: &root,
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
                    path, file_facts, &facts, &resolver,
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

include!("contracts_scope.rs");

#[cfg(test)]
#[path = "contracts_tests.rs"]
mod contracts_tests;
