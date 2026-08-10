use super::*;

#[test]
fn analyzes_project_from_shared_facts_with_absolute_tsconfig() {
    let root = fixture("unique-exports-basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            source: true,
            ..Default::default()
        },
    );
    let tsconfig = root.join("tsconfig.json");

    let findings = analyze_project_with_facts(&root, None, Some(&tsconfig), &facts).unwrap();

    assert_eq!(findings.len(), 2);
}

fn source_file(root: &Path, rel: &str, source: &str) -> SourceFile {
    SourceFile {
        path: root.join(rel),
        rel: rel.to_string(),
        source: source.to_string(),
        symbols: crate::codebase::ts_symbols::extract_symbols(source, false)
            .unwrap()
            .into(),
        disabled: false,
        defer_suppression: false,
        is_nextjs_project: false,
    }
}

fn suppression_origin_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase/unique-exports-suppressed-origin")
}

fn fixture_source_file(root: &Path, rel: &str, defer_suppression: bool) -> SourceFile {
    let source = std::fs::read_to_string(root.join(rel)).unwrap();
    let mut file = source_file(root, rel, &source);
    file.disabled = crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID);
    file.defer_suppression = defer_suppression;
    file
}

fn find_origin(
    target: &Path,
    imported: &str,
    files: &HashMap<PathBuf, SourceFile>,
    resolver: &ImportResolver<'_>,
    workspace: &WorkspaceMap,
) -> ExportOrigin {
    let mut visiting = HashSet::new();
    let remapper =
        crate::codebase::ts_source::FrozenPathRemapper::from_paths(files.keys().cloned());
    collector::find_target_export_origin(
        target,
        imported,
        files,
        resolver,
        workspace,
        &remapper,
        &mut visiting,
    )
    .unwrap()
}

#[test]
fn reexport_origins_cover_resolved_and_fallback_buckets() {
    let root = fixture("unique-exports-edge-cases");
    let reexport_path = root.join("src/origin-test.ts");
    let direct = source_file(
        &root,
        "src/direct.ts",
        "export const Direct = 1;\nexport type DirectType = { id: string };\n",
    );
    let reexport = source_file(
        &root,
        "src/origin-test.ts",
        "export { Direct } from './direct'\n\
export type { DirectType } from './direct'\n\
export { Missing } from './missing'\n\
export type { MissingType } from './missing'\n",
    );
    let files = HashMap::from([
        (direct.path.clone(), direct),
        (reexport.path.clone(), reexport),
    ]);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let workspace = WorkspaceMap::default();

    let direct = find_origin(&reexport_path, "Direct", &files, &resolver, &workspace);
    assert_eq!(direct.file, "src/direct.ts");

    let direct_type = find_origin(&reexport_path, "DirectType", &files, &resolver, &workspace);
    assert_eq!(direct_type.bucket, ExportBucket::Type);

    let missing = find_origin(&reexport_path, "Missing", &files, &resolver, &workspace);
    assert_eq!(missing.file, "src/origin-test.ts");
    assert_eq!(missing.bucket, ExportBucket::Value);

    let missing_type = find_origin(&reexport_path, "MissingType", &files, &resolver, &workspace);
    assert_eq!(missing_type.bucket, ExportBucket::Type);
}

#[test]
fn deferred_suppression_preserves_reexport_origin_identity() {
    let root = suppression_origin_fixture();
    let source = fixture_source_file(&root, "src/source.ts", true);
    let barrel = fixture_source_file(&root, "src/barrel.ts", true);
    let barrel_path = barrel.path.clone();
    let files = HashMap::from([(source.path.clone(), source), (barrel.path.clone(), barrel)]);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);

    let origin = find_origin(
        &barrel_path,
        "Shared",
        &files,
        &resolver,
        &WorkspaceMap::default(),
    );

    assert_eq!(origin.file, "src/source.ts");
}
