use super::*;
use no_mistakes::cli::Format;
use no_mistakes::server_routes::{
    Edge, EdgeKind, Framework, ProjectReport, ServerContractsReport, ServerRoute,
};

fn sample_route() -> ServerRoute {
    ServerRoute {
        file: "src/users.ts".into(),
        line: 1,
        method: "GET".into(),
        route: "/users".into(),
        raw_path: "/users".into(),
        query_params: Vec::new(),
        framework: Framework::Express,
    }
}

fn sample_report() -> ProjectReport {
    ProjectReport {
        summary: Default::default(),
        routes: vec![sample_route()],
        edges: vec![Edge {
            from: "src/users.ts".into(),
            to: "/users".into(),
            kind: EdgeKind::ServerRoute,
        }],
        diagnostics: Vec::new(),
    }
}

#[test]
fn print_helpers_cover_each_format_and_file_filters() {
    let report = sample_report();
    let edges = report.edges.clone();
    for format in [
        Format::Json,
        Format::Yml,
        Format::Md,
        Format::Paths,
        Format::Human,
    ] {
        print_routes(&report, &[], format).unwrap();
        print_routes(&report, &["/users".into(), "missing".into()], format).unwrap();
        print_edges(&edges, format).unwrap();
        print_related(&["src/users.ts".into()], &edges, format).unwrap();
    }
}

#[test]
fn trace_server_analysis_runs_with_and_without_an_observer() {
    let value = trace_server_analysis("analysis.prepare", || Ok::<_, anyhow::Error>(7)).unwrap();
    assert_eq!(value, 7);
    let observer = no_mistakes::diagnostics::InvocationObserver::new(false);
    let guard = no_mistakes::diagnostics::InvocationGuard::install(observer);
    let traced = trace_server_analysis("analysis.server", || Ok::<_, anyhow::Error>(8)).unwrap();
    assert_eq!(traced, 8);
    drop(guard);
}

#[test]
fn print_contracts_covers_each_format() {
    let report = ServerContractsReport {
        routes: vec![],
        client_refs: vec![],
        mismatches: vec![],
    };
    for format in [
        Format::Json,
        Format::Yml,
        Format::Md,
        Format::Paths,
        Format::Human,
    ] {
        print_contracts(&report, format).unwrap();
    }
}

#[test]
fn run_contracts_prints_json_for_a_fixture_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes/remix/fixture");
    let code = run(ServerArgs {
        root,
        tsconfig: None,
        filters: Vec::new(),
        depth: None,
        format: Format::Json,
        json: true,
        timings: false,
        command: ServerCommand::Contracts,
    })
    .expect("contracts command runs");
    assert_eq!(code, ExitCode::SUCCESS);
}
