use crate::codebase::dependencies::Direction;

pub(super) fn graph_direction(report_type: &str) -> Option<Direction> {
    match report_type {
        "dependencies" => Some(Direction::Deps),
        "dependents" | "related" => Some(Direction::Dependents),
        _ => None,
    }
}

pub(super) fn is_symbols_report(report_type: &str) -> bool {
    report_type == "symbols"
}

pub(super) fn is_playwright_report(report_type: &str) -> bool {
    matches!(
        report_type,
        "playwrightCheck" | "playwrightEdges" | "playwrightRelated" | "playwrightTests"
    )
}

pub(super) fn is_project_report(report_type: &str) -> bool {
    matches!(
        report_type,
        "queues"
            | "queueEdges"
            | "queueRelated"
            | "queueCheck"
            | "serverRoutes"
            | "serverRouteList"
            | "serverRouteEdges"
            | "serverRouteRelated"
            | "serverContracts"
            | "reactAnalyze"
            | "reactCheck"
            | "reactUsages"
            | "check"
    )
}
