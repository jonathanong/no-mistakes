use std::path::PathBuf;

/// Checked-in multi-config fixture used to measure request-level resolver
/// construction plus repeated importer-scope selection, separately from
/// repository discovery and parsing.
pub struct ScopedResolverSelectionFixture {
    catalog: crate::codebase::ts_resolver::TsConfigCatalog,
    visible: crate::fx::PathSet,
    importer: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopedResolverSelectionSummary {
    pub resolved: usize,
    pub selection_builds: usize,
}

pub fn scoped_resolver_selection_fixture() -> ScopedResolverSelectionFixture {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution")
        .canonicalize()
        .expect("scoped resolver benchmark fixture should exist");
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &visible_paths,
    );
    ScopedResolverSelectionFixture {
        catalog,
        visible: visible_paths.into_iter().collect(),
        importer: root.join("apps/web/src/entry.ts"),
    }
}

pub fn resolve_repeated_scoped_imports(
    fixture: &ScopedResolverSelectionFixture,
    requests: usize,
) -> ScopedResolverSelectionSummary {
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_visible(
        &fixture.catalog,
        &fixture.visible,
    );
    let resolved = (0..requests)
        .filter(|request| {
            let specifier = if request % 2 == 0 {
                "@runtime/value"
            } else {
                "@shared/message"
            };
            resolver.resolve(specifier, &fixture.importer).is_some()
        })
        .count();
    ScopedResolverSelectionSummary {
        resolved,
        selection_builds: resolver.importer_selection_build_count(),
    }
}

/// Construct scoped resolvers repeatedly against one request-visible universe.
///
/// This keeps resolver construction separate from importer selection so the
/// benchmark catches per-resolver visibility projection work.
pub fn build_repeated_scoped_resolvers(
    fixture: &ScopedResolverSelectionFixture,
    resolvers: usize,
) -> usize {
    (0..resolvers)
        .map(|_| {
            let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_visible(
                &fixture.catalog,
                &fixture.visible,
            );
            usize::from(
                resolver
                    .resolve("@runtime/value", &fixture.importer)
                    .is_some(),
            )
        })
        .sum()
}

/// Build a catalog with a large lexical path list dominated by exact matches.
///
/// The repeated spelling mirrors a monorepo candidate universe where the
/// configured root is already a lexical member, so canonical aliases should
/// remain cold.
pub fn build_catalog_with_large_lexical_visibility(paths: usize) -> usize {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution")
        .canonicalize()
        .expect("catalog benchmark fixture should exist");
    let config = root.join("tsconfig.json");
    let visible = std::iter::repeat_n(config, paths).collect::<Vec<_>>();
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &visible,
    );
    usize::from(
        catalog
            .provenance_for(&root.join("apps/web/src/entry.ts"))
            .config
            .is_some(),
    )
}
