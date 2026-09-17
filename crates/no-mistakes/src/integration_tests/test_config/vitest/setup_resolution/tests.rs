use super::*;
use crate::codebase::ts_resolver::ImportClassification;

struct MissingSourceResolver;

impl ImportResolution for MissingSourceResolver {
    fn resolve(&self, _: &str, _: &Path) -> Option<PathBuf> {
        unreachable!("a missing setup source has no imports to resolve")
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        unreachable!("a missing setup source has no imports to resolve")
    }

    fn visible_files(&self) -> Option<&dyn crate::codebase::ts_resolver::VisiblePathLookup> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &crate::codebase::workspaces::IndexedWorkspaceMap,
        _: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    ) -> ImportClassification {
        unreachable!("a missing setup source has no imports to classify")
    }
}

#[test]
fn deleted_runtime_setup_source_skips_transitive_import_walk() {
    // This fixture deliberately leaves only runtime.d.ts after runtime.ts is
    // deleted. The missing source must not abort runner-config parsing.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-declaration-runtime-deleted");
    let mut candidates = BTreeSet::new();

    runtime_setup_candidates(
        &root.join("setup/runtime.ts"),
        &root,
        &MissingSourceResolver,
        &mut candidates,
    );

    assert!(candidates.is_empty());
}

fn json_import_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check/integration-setup-json-import"),
    )
}

#[test]
fn json_setup_import_is_never_parsed() {
    // Pin the invariant fix 2 protects: a resolved JSON specifier (e.g.
    // `with { type: 'json' }`) short-circuits before `read_request_source`/
    // `with_program`, so it never reaches the resolver-call branches guarded
    // by this test's `MissingSourceResolver` (its methods are `unreachable!`).
    // `.json` also fails to parse on its own (`unsupported JavaScript/
    // TypeScript file`), so this alone would not panic without fix 2 either;
    // fix 2's real payoff is skipping a wasted read/parse and a poisoned
    // request-scoped `ParsedProgramCache` entry (see `cache.rs` fix 1, which
    // is the fix that a regression here would actually be masking).
    let root = json_import_fixture_root();
    let mut candidates = BTreeSet::new();

    runtime_setup_candidates(
        &root.join("localization/catalog/aliases.json"),
        &root,
        &MissingSourceResolver,
        &mut candidates,
    );

    assert!(candidates.is_empty());
}

#[test]
fn json_setup_import_stays_a_trigger_candidate_through_a_real_resolver() {
    // Walking the real setup file must still record the JSON catalog as a
    // deletion-trigger candidate (recall is preserved) even though fix 2
    // never parses it. `resolution_candidates` records the candidate before
    // `collect_runtime_setup_candidates` ever inspects the extension.
    let root = json_import_fixture_root();
    let tsconfig = crate::integration_tests::test_support::tsconfig_without_config(&root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let mut candidates = BTreeSet::new();

    runtime_setup_candidates(
        &root.join("setup/vitest.setup.mts"),
        &root,
        &resolver,
        &mut candidates,
    );

    assert!(
        candidates.contains(&root.join("localization/catalog/aliases.json")),
        "{candidates:#?}"
    );
}
