use crate::report::types::{
    ApiCallOccurrence, DuplicateApiCall, FetchSide, FinalReport, RouteReport, Summary,
    UnsupportedApiCall,
};
use std::collections::{HashMap, HashSet};

pub(crate) fn build_final_report(reports: Vec<RouteReport>) -> FinalReport {
    let total_routes = reports.len();
    let routes_with_api_calls = reports
        .iter()
        .filter(|route| !route.api_calls.is_empty())
        .count();

    let mut duplicate_key_map: HashMap<(String, String, FetchSide, bool), Vec<ApiCallOccurrence>> =
        HashMap::new();
    let mut unique_api_calls = HashSet::new();
    let mut dynamic_api_calls = 0usize;
    let mut cached_api_calls = 0usize;
    let mut client_api_calls = 0usize;
    let mut server_api_calls = 0usize;
    let mut rsc_api_calls = 0usize;
    let mut unsupported = Vec::new();

    for route in &reports {
        for api_call in &route.api_calls {
            let key = (
                api_call.method.clone(),
                api_call.path.clone(),
                api_call.side.clone(),
                api_call.rsc,
            );
            duplicate_key_map
                .entry(key)
                .or_default()
                .push(ApiCallOccurrence {
                    route: route.route.clone(),
                    file: api_call.file.clone(),
                    line: api_call.line,
                });

            unique_api_calls.insert((
                api_call.method.clone(),
                api_call.path.clone(),
                api_call.side.clone(),
            ));

            if api_call.dynamic {
                dynamic_api_calls += 1;
                unsupported.push(UnsupportedApiCall {
                    route: route.route.clone(),
                    file: api_call.file.clone(),
                    line: api_call.line,
                    reason: "dynamic-path".to_string(),
                    raw_path: api_call.raw_path.clone(),
                });
            }
            if api_call.cached {
                cached_api_calls += 1;
            }
            match api_call.side {
                FetchSide::Client => client_api_calls += 1,
                FetchSide::Server => server_api_calls += 1,
            }
            if api_call.rsc {
                rsc_api_calls += 1;
            }
        }
    }

    let mut duplicates = Vec::new();
    for ((method, path, side, rsc), occurrences) in duplicate_key_map {
        if occurrences.len() > 1 {
            duplicates.push(DuplicateApiCall {
                key: format!(
                    "{method} {path} {} {}",
                    match side {
                        FetchSide::Client => "client",
                        FetchSide::Server => "server",
                    },
                    if rsc { "rsc" } else { "non-rsc" }
                ),
                count: occurrences.len(),
                occurrences,
            });
        }
    }

    let duplicate_api_calls: usize = duplicates
        .iter()
        .map(|entry| entry.count.saturating_sub(1))
        .sum();

    let mut final_report = FinalReport {
        summary: Summary {
            total_routes,
            routes_with_api_calls,
            total_api_calls: reports.iter().map(|route| route.api_calls.len()).sum(),
            unique_api_calls: unique_api_calls.len(),
            duplicate_api_calls,
            dynamic_api_calls,
            cached_api_calls,
            client_api_calls,
            server_api_calls,
            rsc_api_calls,
        },
        routes: reports,
        duplicates,
        unsupported,
    };

    final_report.unsupported.sort_by(|a, b| {
        a.route
            .cmp(&b.route)
            .then(a.file.cmp(&b.file))
            .then(a.line.cmp(&b.line))
    });
    final_report
        .duplicates
        .sort_by(|a, b| a.key.cmp(&b.key).then(a.count.cmp(&b.count)));

    final_report
}
