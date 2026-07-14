use super::model::{package_name_from_specifier, ImportUsage, ImportUsageFile, ImportUsagesReport};
use super::*;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/import-usages/fixture"),
    )
}

fn base_args() -> ImportUsagesArgs {
    ImportUsagesArgs {
        files: Vec::new(),
        root: Some(fixture_root()),
        scan_roots: Vec::new(),
        filters: Vec::new(),
        format: Some(Format::Json),
        json: false,
        timings: false,
    }
}

fn sample_report() -> ImportUsagesReport {
    ImportUsagesReport {
        roots: vec!["fixture".to_string()],
        files: vec![
            ImportUsageFile {
                path: "src/empty.mts".to_string(),
                imports: vec![],
            },
            ImportUsageFile {
                path: "src/main.mts".to_string(),
                imports: vec![ImportUsage {
                    specifier: "react".to_string(),
                    package_name: Some("react".to_string()),
                    kind: "static",
                    line: 1,
                    side_effect_only: false,
                    re_export: false,
                }],
            },
        ],
    }
}

#[test]
fn run_covers_cli_output_and_timing_path() {
    let mut cli_args = base_args();
    cli_args.files = vec![PathBuf::from("src/main.mts")];
    cli_args.format = Some(Format::Paths);
    cli_args.timings = true;

    run(cli_args).unwrap();
}

#[test]
fn output_formats_cover_all_branches() {
    let mut report = sample_report();
    report.files[1].imports.push(ImportUsage {
        specifier: "./local".to_string(),
        package_name: None,
        kind: "static",
        line: 2,
        side_effect_only: false,
        re_export: false,
    });
    assert_eq!(output::resolve_format(true, None, false), Format::Json);
    assert_eq!(
        output::resolve_format(false, Some(Format::Yml), false),
        Format::Yml
    );
    assert_eq!(output::resolve_format(false, None, true), Format::Human);
    assert_eq!(output::resolve_format(false, None, false), Format::Json);

    for format in [Format::Json, Format::Yml, Format::Human, Format::Paths] {
        let mut out = Vec::new();
        output::write_report(&report, format, &mut out).unwrap();
        assert!(!out.is_empty());
    }
}

#[test]
fn scan_roots_filters_and_output_roots_cover_path_branches() {
    let cwd = std::env::current_dir().unwrap();
    let root = fixture_root();
    let mut scan_args = base_args();
    scan_args.scan_roots = vec![PathBuf::from(".")];
    scan_args.filters = vec!["src/".to_string()];

    let files = paths::resolve_files(&scan_args, &root, &cwd).unwrap();
    assert!(files.iter().any(|path| path.ends_with("src/main.mts")));
    assert_eq!(paths::roots_for_output(&scan_args, &root), vec!["."]);

    let mut absolute_scan_args = base_args();
    absolute_scan_args.scan_roots = vec![root.join("src")];
    let absolute_files = paths::resolve_files(&absolute_scan_args, &root, &cwd).unwrap();
    assert!(absolute_files
        .iter()
        .any(|path| path.ends_with("src/main.mts")));

    let mut file_args = base_args();
    file_args.files = vec![PathBuf::from("src/main.mts")];
    assert_eq!(
        paths::roots_for_output(&file_args, &root),
        vec!["src/main.mts"]
    );

    let mut absolute_file_args = base_args();
    absolute_file_args.files = vec![root.join("src/main.mts")];
    let absolute_input = paths::resolve_files(&absolute_file_args, &root, &cwd).unwrap();
    assert_eq!(absolute_input, vec![root.join("src/main.mts")]);
}

#[test]
fn session_scan_roots_reuse_inside_snapshot_and_discover_external_root() {
    let cwd = std::env::current_dir().unwrap();
    let root = fixture_root();
    let external_root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/signature-impact"),
    );
    let root_main = root.join("src/main.mts");
    let external_consumer = external_root.join("consumer.mts");
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let mut args = base_args();
    args.scan_roots = vec![PathBuf::from("src"), root.join("src"), external_root];

    let files = paths::resolve_files_with_session(&session, &args, &root, &cwd).unwrap();

    assert_eq!(files.iter().filter(|path| *path == &root_main).count(), 1);
    assert!(files.contains(&external_consumer));
}

#[test]
fn relative_roots_and_empty_package_segments_are_handled() {
    let mut file_args = base_args();
    file_args.files = vec![PathBuf::from("src/main.mts")];
    let files = paths::resolve_files(&file_args, Path::new("."), &fixture_root()).unwrap();

    assert_eq!(paths::normalize_root(None, &fixture_root()), fixture_root());
    assert!(files[0].ends_with("src/main.mts"));
    assert_eq!(package_name_from_specifier(""), None);
    assert_eq!(package_name_from_specifier("@scope/"), None);
}

struct NoFacts;

impl crate::codebase::dependencies::graph::TsFactLookup for NoFacts {
    fn get_ts_facts(
        &self,
        _path: &Path,
    ) -> Option<&crate::codebase::ts_source::facts::TsFileFacts> {
        None
    }
}

#[test]
fn collect_from_facts_skips_files_without_facts() {
    let root = fixture_root();
    let graph_files = GraphFiles::from_files(vec![root.join("src/main.mts")]);
    let report =
        collect_from_facts(&root, vec!["fixture".to_string()], &graph_files, &NoFacts).unwrap();

    assert!(report.files.is_empty());
}
