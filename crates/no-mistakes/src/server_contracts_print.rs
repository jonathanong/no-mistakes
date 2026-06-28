fn print_contracts(
    report: &no_mistakes::server_routes::ServerContractsReport,
    format: Format,
) -> Result<()> {
    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(report)?),
        Format::Yml => println!("{}", serde_yaml::to_string(report)?),
        Format::Md => {
            println!("# Server contracts");
            for route in &report.routes {
                println!(
                    "- `{}` {} `{}` query: {}",
                    route.file,
                    route.method,
                    route.route,
                    route.query_params.join(", ")
                );
            }
            for mismatch in &report.mismatches {
                println!(
                    "- mismatch `{}` -> `{}` missing `{}`",
                    mismatch.file,
                    mismatch.matched_route,
                    mismatch.missing_params.join(", ")
                );
            }
        }
        Format::Paths => {
            let paths: BTreeSet<&str> = report
                .routes
                .iter()
                .map(|route| route.file.as_str())
                .chain(
                    report
                        .client_refs
                        .iter()
                        .map(|route_ref| route_ref.file.as_str()),
                )
                .collect();
            for path in paths {
                println!("{path}");
            }
        }
        Format::Human => {
            for route in &report.routes {
                println!(
                    "{} {} -> {} query: {}",
                    route.method,
                    route.file,
                    route.route,
                    route.query_params.join(", ")
                );
            }
            for mismatch in &report.mismatches {
                println!(
                    "warning: {}:{} {} uses query params not read by {}: {}",
                    mismatch.file,
                    mismatch.line,
                    mismatch.route,
                    mismatch.matched_route,
                    mismatch.missing_params.join(", ")
                );
            }
        }
    }
    Ok(())
}
