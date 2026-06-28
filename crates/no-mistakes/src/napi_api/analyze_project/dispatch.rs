use crate::codebase::dependencies::Direction;

type ReportRunner = fn(String) -> napi::Result<String>;

pub(super) fn graph_direction(report_type: &str) -> Option<Direction> {
    match report_type {
        "dependencies" => Some(Direction::Deps),
        "dependents" | "related" => Some(Direction::Dependents),
        _ => None,
    }
}

pub(super) fn symbols_runner(report_type: &str) -> Option<ReportRunner> {
    match report_type {
        "symbols" => Some(super::super::symbols_json_impl),
        _ => None,
    }
}

pub(super) fn playwright_runner(report_type: &str) -> Option<ReportRunner> {
    match report_type {
        "playwrightCheck" => Some(super::super::playwright_check_json_impl),
        "playwrightEdges" => Some(super::super::playwright_edges_json_impl),
        "playwrightRelated" => Some(super::super::playwright_related_json_impl),
        "playwrightTests" => Some(super::super::playwright_tests_json_impl),
        _ => None,
    }
}

pub(super) fn project_runner(report_type: &str) -> Option<ReportRunner> {
    match report_type {
        "queues" => Some(super::super::queues_json_impl),
        "queueEdges" => Some(super::super::queue_edges_json_impl),
        "queueRelated" => Some(super::super::queue_related_json_impl),
        "queueCheck" => Some(super::super::queue_check_json_impl),
        "serverRoutes" => Some(super::super::server_routes_json_impl),
        "serverRouteList" => Some(super::super::server_route_list_json_impl),
        "serverRouteEdges" => Some(super::super::server_route_edges_json_impl),
        "serverRouteRelated" => Some(super::super::server_route_related_json_impl),
        "serverContracts" => Some(super::super::server_contracts_json_impl),
        "reactAnalyze" => Some(super::super::react_analyze_json_impl),
        "reactCheck" => Some(super::super::react_check_json_impl),
        "reactUsages" => Some(super::super::react_usages_json_impl),
        "check" => Some(super::super::check_json_impl),
        _ => None,
    }
}
