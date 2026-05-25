use crate::fetches::report::types::{
    CacheKind, FetchOccurrence, FetchSide, FinalReport, SourceType,
};

pub(crate) fn cache_kind_name(cache_kind: &CacheKind) -> &'static str {
    match cache_kind {
        CacheKind::None => "none",
        CacheKind::FetchCache => "fetch-cache",
        CacheKind::FetchNextRevalidate => "fetch-next-revalidate",
        CacheKind::FetchNextTags => "fetch-next-tags",
        CacheKind::ReactCache => "react-cache",
        CacheKind::Cache => "cache",
        CacheKind::UnstableCache => "unstable-cache",
    }
}

pub(crate) fn fetch_cache_label(fetch: &FetchOccurrence) -> String {
    if !fetch.cached {
        return "no".to_string();
    }

    let kind = cache_kind_name(&fetch.cache_kind);
    match &fetch.cached_function {
        Some(cached_function) => format!("{kind} ({cached_function})"),
        None => kind.to_string(),
    }
}

pub(crate) fn print_markdown_report(report: &FinalReport) {
    println!("# Next.js Fetch API Analysis");
    println!();
    println!("## Summary");
    println!("- Total Routes: {}", report.summary.total_routes);
    println!(
        "- Routes with API Calls: {}",
        report.summary.routes_with_api_calls
    );
    println!("- Total API Calls: {}", report.summary.total_api_calls);
    println!("- Unique API Calls: {}", report.summary.unique_api_calls);
    println!(
        "- Duplicate API Calls: {}",
        report.summary.duplicate_api_calls
    );
    println!("- Dynamic API Calls: {}", report.summary.dynamic_api_calls);
    println!("- Cached API Calls: {}", report.summary.cached_api_calls);
    println!("- Server API Calls: {}", report.summary.server_api_calls);
    println!("- RSC API Calls: {}", report.summary.rsc_api_calls);
    println!("- Client API Calls: {}", report.summary.client_api_calls);
    println!(
        "- Conditional API Calls: {}",
        report.summary.conditional_api_calls
    );
    println!(
        "- Parallel API Calls: {}",
        report.summary.parallel_api_calls
    );
    println!(
        "- Error-Handled API Calls: {}",
        report.summary.error_handled_api_calls
    );
    println!();

    println!("## Routes");
    for route in &report.routes {
        println!("### {} ({})", route.route, route.file);
        if route.api_calls.is_empty() {
            println!("(no fetches found)");
        } else {
            println!("| Method | Path | Side | File | Line | RSC | Dynamic | Cache | Function | Cond | P.all | ErrHandled | Source |");
            println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
            let mut unique_fetches = route.api_calls.clone();
            unique_fetches.sort();
            unique_fetches.dedup();
            for fetch in &unique_fetches {
                println!(
                    "| {} | `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    fetch.method,
                    fetch.path,
                    if matches!(fetch.side, FetchSide::Client) {
                        "client"
                    } else {
                        "server"
                    },
                    fetch.file,
                    fetch.line,
                    if fetch.rsc { "yes" } else { "no" },
                    if fetch.dynamic { "✅" } else { "❌" },
                    fetch_cache_label(fetch),
                    fetch.function_name.as_deref().unwrap_or("-"),
                    if fetch.conditional { "yes" } else { "no" },
                    if fetch.in_promise_all { "yes" } else { "no" },
                    if fetch.error_handled { "yes" } else { "no" },
                    source_type_label(&fetch.source_type),
                );
            }
        }
        println!();
    }

    if !report.duplicates.is_empty() {
        println!("## Duplicates");
        println!("| Key | Count | Route | File | Line |");
        println!("| --- | --- | --- | --- | --- |");
        for fetch in &report.duplicates {
            for occurrence in &fetch.occurrences {
                println!(
                    "| `{}` | {} | {} | {} | {} |",
                    fetch.key, fetch.count, occurrence.route, occurrence.file, occurrence.line
                );
            }
        }
        println!();
    }

    if !report.unsupported.is_empty() {
        println!("## Unsupported (Dynamic)");
        println!("| Route | File | Line | Reason | Path |");
        println!("| --- | --- | --- | --- | --- |");
        for fetch in &report.unsupported {
            println!(
                "| {} | {} | {} | {} | `{}` |",
                fetch.route, fetch.file, fetch.line, fetch.reason, fetch.raw_path
            );
        }
        println!();
    }
}

fn source_type_label(source_type: &SourceType) -> &'static str {
    match source_type {
        SourceType::Page => "page",
        SourceType::Layout => "layout",
        SourceType::Loading => "loading",
        SourceType::Error => "error",
        SourceType::Template => "template",
        SourceType::Route => "route",
        SourceType::Module => "module",
    }
}
