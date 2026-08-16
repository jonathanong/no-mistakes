fn graph_files_source_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature {signature}"));
    let brace = start
        + source[start..]
            .find('{')
            .unwrap_or_else(|| panic!("missing body for {signature}"));
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[brace..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[brace..=brace + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated body for {signature}")
}

#[test]
fn graph_files_constructor_does_not_eagerly_canonicalize() {
    let source = include_str!("../graph_files.rs");
    let constructor = graph_files_source_function_body(
        source,
        "pub(crate) fn from_files_with_resource_candidates_excluding_indexable(",
    );
    assert!(
        !constructor.contains("canonicalize"),
        "from_files must not realpath every visible path"
    );
    let visible_path = graph_files_source_function_body(source, "pub(crate) fn visible_path(");
    assert!(
        visible_path.contains("canonicalize"),
        "visible_path must keep the lazy canonicalize fallback"
    );
}

#[test]
fn playwright_fact_analysis_forwards_prepared_graph_files() {
    for (source, signature) in [
        (
            include_str!("../../../../playwright/analysis/pipeline_entrypoints.rs"),
            "pub(crate) fn analyze_with_policy_from_snapshot(",
        ),
        (
            include_str!("../../../../playwright/analysis/pipeline_entrypoints.rs"),
            "pub(crate) fn analyze_with_policy_and_facts_from_snapshot(",
        ),
        (
            include_str!("../../../../playwright/analysis/pipeline_selectors.rs"),
            "pub(crate) fn analyze_selectors_with_policy_from_snapshot(",
        ),
        (
            include_str!("../../../../playwright/analysis/pipeline_selectors.rs"),
            "pub(crate) fn analyze_selectors_with_policy_and_facts_from_snapshot(",
        ),
    ] {
        let body = graph_files_source_function_body(source, signature);
        assert!(
            body.contains("facts.graph_files()"),
            "{signature} must forward the prepared graph-file universe"
        );
        assert!(
            !body.contains("graph_file_universe: None"),
            "{signature} must not drop the prepared graph-file universe"
        );
    }
}

#[test]
fn graph_files_visible_path_hits_discovery_spelling_without_remap() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/selector-text-sparse-universe/fixture"),
    );
    let page = root.join("web/app/page.tsx");
    let files = GraphFiles::from_files(vec![page.clone()]);
    assert_eq!(files.visible_path(&page), Some(page.as_path()));
}

#[cfg(unix)]
#[test]
fn graph_files_visible_path_remaps_canonical_symlink_target() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/symlink-workspace/link");
    let via_link = crate::codebase::ts_resolver::normalize_path(&root.join("src/value.ts"));
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let files = GraphFiles::from_files(vec![via_link.clone()]);
    assert_eq!(files.visible_path(&via_link), Some(via_link.as_path()));
    assert_eq!(files.visible_path(&via_real), Some(via_link.as_path()));
}

#[cfg(unix)]
#[test]
fn graph_files_visible_path_prefers_first_sorted_alias_on_canonical_collision() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/graph-files-dual-alias/fixture"),
    );
    let alias_a = root.join("a.ts");
    let alias_b = root.join("b.ts");
    let target = root.join("target.ts");
    let files = GraphFiles::from_files(vec![alias_b.clone(), alias_a.clone()]);
    assert_eq!(files.visible_path(&alias_a), Some(alias_a.as_path()));
    assert_eq!(files.visible_path(&alias_b), Some(alias_b.as_path()));
    // Target is not in the visible set; the reverse map must pick the first
    // sorted alias rather than HashSet visit order.
    assert_eq!(files.visible_path(&target), Some(alias_a.as_path()));
}

#[cfg(unix)]
#[test]
fn graph_files_keeps_uncanonicalizable_paths_under_discovery_spelling() {
    // Tracked broken symlink: canonicalize fails, discovery spelling stays visible.
    let broken = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/tests-impact/fixture/broken.test.mts");
    let files = GraphFiles::from_files(vec![broken.clone()]);
    assert_eq!(files.visible_path(&broken), Some(broken.as_path()));
    assert!(broken.canonicalize().is_err());
}
