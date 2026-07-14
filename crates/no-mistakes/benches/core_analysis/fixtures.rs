use clap::Parser;
use no_mistakes::codebase::dependencies::{RelationshipArg, TraverseArgs, TsConfig};
use no_mistakes::codebase::ts_source::discover_visible_paths;
use no_mistakes::impacted_checks::ImpactedChecksArgs;
use std::path::{Path, PathBuf};

pub(super) const EXPECTED_SOURCE_FILES: usize = 14;
pub(super) const EXPECTED_IMPACTED_CHECKS: usize = 1;
pub(super) const EXPECTED_MULTI_REPORT_RESOLVER_KEYS: u64 = 8;
pub(super) const EXPECTED_CHECK_SOURCE_READS: u64 = 16;
pub(super) const EXPECTED_CHECK_MANIFEST_PARSES: u64 = 4;
pub(super) const EXPECTED_CHECK_RESOLVER_KEYS: u64 = 12;

pub(super) fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/core-analysis")
        .canonicalize()
        .expect("performance fixture should exist")
}

pub(super) fn tsconfig(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths: vec![(
            "@core/*".to_string(),
            vec!["packages/core/src/*".to_string()],
        )],
        paths_dir: root.to_path_buf(),
        base_url: Some(root.to_path_buf()),
    }
}

pub(super) fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = discover_visible_paths(root)
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ts" | "tsx" | "mts")
            )
        })
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files.len(), EXPECTED_SOURCE_FILES);
    files
}

pub(super) fn traverse_args(
    root: &Path,
    roots: &[&str],
    relationship: RelationshipArg,
) -> TraverseArgs {
    TraverseArgs {
        file_symbols: vec![None; roots.len()],
        file_entrypoints_are_structured: vec![false; roots.len()],
        root: Some(root.to_path_buf()),
        tsconfig: Some(root.join("tsconfig.json")),
        depth: None,
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        format: None,
        json: true,
        relationships: vec![relationship],
        include_symbols: false,
        timings: false,
        files: roots.iter().map(PathBuf::from).collect(),
    }
}

#[derive(Parser)]
struct ImpactedBenchArgs {
    #[command(flatten)]
    args: ImpactedChecksArgs,
}

pub(super) fn impacted_args(root: &Path) -> ImpactedChecksArgs {
    ImpactedBenchArgs::try_parse_from([
        "impacted-bench",
        "--root",
        root.to_str().expect("fixture root should be UTF-8"),
        "--config",
        root.join(".no-mistakes.yml")
            .to_str()
            .expect("fixture config should be UTF-8"),
        "src/app.tsx",
    ])
    .expect("benchmark arguments should parse")
    .args
}
