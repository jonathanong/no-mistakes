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
    let visible_path_source = include_str!("../graph_files_visible.rs");
    let visible_path =
        graph_files_source_function_body(visible_path_source, "pub(crate) fn visible_path(");
    assert!(
        visible_path.contains("canonicalize"),
        "visible_path must keep the lazy canonicalize fallback"
    );
}

#[test]
fn graph_files_visible_does_not_build_a_pathbuf_hashset() {
    let source = include_str!("../graph_files.rs");
    assert!(
        source.contains("visible: Vec<u8>"),
        "GraphFiles must store visible membership as a dense bitset"
    );
    let constructor = graph_files_source_function_body(
        source,
        "pub(crate) fn from_files_with_resource_candidates_excluding_indexable(",
    );
    assert!(
        constructor.contains("vec![1u8; all.len()]"),
        "from_files must mark visibility with a parallel bitset, not a cloned path set"
    );
    assert!(
        constructor.contains("as_os_str().cmp") && !constructor.contains("all.sort();"),
        "from_files must sort visible paths with OsStr order, not Path::cmp"
    );
    assert!(
        !constructor.contains("visible:")
            || constructor.contains("let visible = vec![1u8; all.len()]"),
        "from_files must not assign a HashSet to visible"
    );
    let visible_source = include_str!("../graph_files_visible.rs");
    assert!(
        !visible_source.contains("HashSet<PathBuf>")
            && !visible_source.contains("FxHashSet<PathBuf>"),
        "visible lookup must not clone paths into a HashSet"
    );
    let trait_contains = graph_files_source_function_body(
        visible_source,
        "impl crate::codebase::ts_resolver::VisiblePathLookup for GraphFiles",
    );
    assert!(
        trait_contains.contains("GraphFiles::contains_visible(self, path)")
            && !trait_contains.contains("self.visible_path"),
        "VisiblePathLookup::contains_visible must stay exact HashSet-style membership"
    );
    let visible_index = graph_files_source_function_body(
        visible_source,
        "fn visible_index(&self, path: &Path) -> Option<usize>",
    );
    assert!(
        visible_index.contains("as_os_str().cmp")
            && !visible_index.contains("as_path().cmp")
            && !visible_index.contains("canonicalize"),
        "visible_index must probe OsStr bytes without Path::cmp or canonicalize"
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
fn graph_files_explicit_root_keeps_first_sorted_canonical_alias() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/graph-files-dual-alias/fixture"),
    );
    let alias_a = root.join("a.ts");
    let alias_b = root.join("b.ts");
    let target = root.join("target.ts");

    let mut later_then_earlier = GraphFiles::from_files(vec![alias_b.clone()]);
    assert_eq!(
        later_then_earlier.visible_path(&target),
        Some(alias_b.as_path())
    );
    assert!(later_then_earlier.add_explicit_root(&alias_a));
    assert_eq!(
        later_then_earlier.visible_path(&target),
        Some(alias_a.as_path())
    );

    let mut earlier_then_later = GraphFiles::from_files(vec![alias_a.clone()]);
    assert_eq!(
        earlier_then_later.visible_path(&target),
        Some(alias_a.as_path())
    );
    assert!(earlier_then_later.add_explicit_root(&alias_b));
    assert_eq!(
        earlier_then_later.visible_path(&target),
        Some(alias_a.as_path())
    );
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

#[cfg(unix)]
#[test]
fn graph_files_visible_path_uses_canonical_spelling_already_in_visible() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/symlink-workspace/link");
    let via_link = crate::codebase::ts_resolver::normalize_path(&root.join("src/value.ts"));
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let files = GraphFiles::from_files(vec![via_real.clone()]);
    assert_eq!(files.visible_path(&via_link), Some(via_real.as_path()));
}

#[cfg(unix)]
#[test]
fn graph_files_explicit_root_inserts_unrelated_canonical_key() {
    let alias_root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/graph-files-dual-alias/fixture"),
    );
    let page = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/codebase/dependencies/selector-text-sparse-universe/fixture/web/app/page.tsx",
        ),
    );
    let alias_a = alias_root.join("a.ts");
    let target = alias_root.join("target.ts");
    let mut files = GraphFiles::from_files(vec![alias_a.clone()]);
    assert_eq!(files.visible_path(&target), Some(alias_a.as_path()));
    assert!(files.add_explicit_root(&page));
    assert_eq!(files.visible_path(&page), Some(page.as_path()));
    assert_eq!(files.visible_path(&target), Some(alias_a.as_path()));
}

#[cfg(unix)]
#[test]
fn graph_files_builds_reverse_map_skipping_uncanonicalizable_entries() {
    let page = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/codebase/dependencies/selector-text-sparse-universe/fixture/web/app/page.tsx",
        ),
    );
    let broken = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/tests-impact/fixture/broken.test.mts");
    let via_link = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link/src/value.ts"),
    );
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let files = GraphFiles::from_files(vec![broken, via_link.clone(), page]);
    assert_eq!(files.visible_path(&via_real), Some(via_link.as_path()));
}

#[test]
fn graph_files_visible_path_is_safe_for_concurrent_lookups() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/selector-text-sparse-universe/fixture"),
    );
    let page = root.join("web/app/page.tsx");
    let files = GraphFiles::from_files(vec![page.clone()]);
    std::thread::scope(|scope| {
        for _ in 0..16 {
            let files = &files;
            let page = &page;
            scope.spawn(move || {
                assert_eq!(files.visible_path(page), Some(page.as_path()));
            });
        }
    });
}

#[test]
fn graph_files_explicit_root_marks_existing_hidden_path_visible() {
    let page = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/codebase/dependencies/selector-text-sparse-universe/fixture/web/app/page.tsx",
        ),
    );
    let mut files = GraphFiles::from_parts(vec![page.clone()], vec![], vec![], vec![]);
    assert!(!files.contains_visible(&page));
    assert!(files.add_explicit_root(&page));
    assert!(files.contains_visible(&page));
    assert!(!files.add_explicit_root(&page));
}

#[test]
fn graph_files_visible_probe_is_os_str_byte_identity() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/ts-source/normalized-path-membership"),
    );
    let nested = root.join("a/b.ts");
    let dashed = root.join("a-b.ts");
    let dotted = root.join("a/./b.ts");
    let collapsed = crate::codebase::ts_resolver::normalize_path(&dotted);

    let files = GraphFiles::from_files(vec![nested.clone(), dashed.clone()]);
    // OsStr bytes put `a-b.ts` before `a/b.ts` (`-` < `/`). Path::cmp would
    // reverse that because it compares the `a` component first.
    assert_eq!(files.all(), [dashed.clone(), nested.clone()]);
    assert_eq!(nested.as_os_str(), collapsed.as_os_str());
    assert_ne!(dotted.as_os_str(), nested.as_os_str());
    assert!(files.contains_visible(&nested));
    assert!(files.contains_visible(&dashed));
    assert!(!files.contains_visible(&dotted));
    assert!(files.contains_visible(&collapsed));
}

#[cfg(unix)]
#[test]
fn graph_files_trait_contains_visible_is_exact_membership() {
    use crate::codebase::ts_resolver::VisiblePathLookup;

    let via_link = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link/src/value.ts"),
    );
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let files = GraphFiles::from_files(vec![via_link.clone()]);

    assert!(VisiblePathLookup::contains_visible(&files, &via_link));
    assert!(!VisiblePathLookup::contains_visible(&files, &via_real));
    assert_eq!(files.visible_path(&via_real), Some(via_link.as_path()));
    assert!(same_graph_universe(std::slice::from_ref(&via_link), &files));
    assert!(!same_graph_universe(
        std::slice::from_ref(&via_real),
        &files
    ));
}

#[cfg(unix)]
#[test]
fn scoped_visibility_projection_reuses_its_arc_and_invalidates_after_explicit_root() {
    use crate::codebase::ts_resolver::VisiblePathLookup;

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let via_link = root.join("src/value.ts");
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let explicit = root.join("src/entry.ts");
    let mut files = GraphFiles::from_files(vec![via_link.clone()]);

    let first = VisiblePathLookup::normalized_visible(&files);
    let second = VisiblePathLookup::normalized_visible(&files);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert!(first.contains(&via_link));
    assert!(first.contains(&via_real));

    assert!(files.add_explicit_root(&explicit));
    let refreshed = VisiblePathLookup::normalized_visible(&files);
    assert!(!std::sync::Arc::ptr_eq(&first, &refreshed));
    assert!(refreshed.contains(&explicit));
}

#[cfg(unix)]
#[test]
fn scoped_session_visibility_keeps_canonical_aliases_from_graph_files() {
    use crate::codebase::ts_resolver::ImportResolution;

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let via_link = crate::codebase::ts_resolver::normalize_path(&root.join("src/value.ts"));
    let via_real = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/value.ts"),
    );
    let files = GraphFiles::from_files(vec![via_link.clone()]);
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json"))
        .expect("symlink workspace tsconfig");
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let resolvers = [
        crate::codebase::ts_resolver::ScopedImportResolver::new_in_session(
            &catalog,
            &files,
            session.as_ref(),
        ),
        crate::codebase::ts_resolver::ScopedImportResolver::from_lookup(
            &catalog,
            &files,
            Some(session.as_ref()),
        ),
    ];

    assert!(!files.contains_visible(&via_real));
    for resolver in resolvers {
        let visible = ImportResolution::visible_files(&resolver)
            .expect("session resolver has a visible universe");
        assert!(visible.contains_visible(&via_link));
        assert!(
            visible.contains_visible(&via_real),
            "scoped resolvers must own canonical aliases, not GraphFiles exact membership"
        );
    }
}
