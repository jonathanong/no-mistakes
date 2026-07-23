use super::*;
use crate::codebase::ts_resolver::{ImportClassification, ImportResolution};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

struct UnreadableExtendsResolver {
    path: PathBuf,
}

impl ImportResolution for UnreadableExtendsResolver {
    fn resolve(&self, specifier: &str, _: &Path) -> Option<PathBuf> {
        (specifier == "./unreadable.js").then(|| self.path.clone())
    }

    fn resolution_candidates(&self, specifier: &str, _: &Path) -> BTreeSet<PathBuf> {
        self.resolve(specifier, Path::new("")).into_iter().collect()
    }

    fn visible_files(&self) -> Option<&HashSet<PathBuf>> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &crate::codebase::workspaces::IndexedWorkspaceMap,
        _: &HashSet<PathBuf>,
    ) -> ImportClassification {
        unreachable!("config extends parsing only resolves the literal extends source")
    }
}

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn unreadable_static_vitest_config_extends_keeps_owner_fallback() {
    let fixture = saved_fixture("vitest-extends-read-error");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let resolver = UnreadableExtendsResolver {
        path: root.join("unreadable.js"),
    };
    let projects =
        crate::integration_tests::runner_config::with_program(&path, &source, |program, source| {
            crate::integration_tests::test_config::vitest::parse_program_with_resolver(
                program, source, &path, &root, &root, &resolver,
            )
        })
        .unwrap()
        .unwrap();

    let setups = &projects[0].vitest_setup;
    assert_eq!(setups.len(), 2, "{setups:#?}");
    let provenance = setups
        .iter()
        .find(|setup| setup.config_extends_provenance)
        .expect("a resolved-but-unreadable config remains a config-change trigger");
    assert!(provenance.unresolved_config_extends.is_none());
    assert!(provenance.trigger_paths.contains(&resolver.path));

    let fallback = setups
        .iter()
        .find(|setup| setup.unresolved_config_extends.is_some())
        .expect("an unreadable config keeps the owner-scoped conservative fallback");
    assert_eq!(
        fallback.unresolved_config_extends.as_deref(),
        Some("./unreadable.js")
    );
    assert!(!fallback.config_extends_provenance);
    assert!(fallback.trigger_paths.contains(&path));
    assert!(fallback.trigger_paths.contains(&resolver.path));
}

#[test]
fn absolute_static_vitest_config_extends_keeps_config_provenance() {
    let fixture = saved_fixture("vitest-extends-absolute");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let base = root.join("base.js");
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("__ABSOLUTE_EXTENDS__", &base.to_string_lossy());
    let project = parse_vitest_fixture(&source, &path, &root)
        .unwrap()
        .remove(0);

    assert!(project.vitest_setup[0].config_extends_provenance);
    assert!(project.vitest_setup[0].trigger_paths.contains(&base));
}

#[test]
fn unresolved_absolute_static_vitest_config_extends_keeps_candidate_trigger() {
    let fixture = saved_fixture("vitest-extends-absolute-unresolved");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let missing = root.join("missing.js");
    let missing_source = missing.to_string_lossy().into_owned();
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("__ABSOLUTE_UNRESOLVED_EXTENDS__", &missing_source);
    let project = parse_vitest_fixture(&source, &path, &root)
        .unwrap()
        .remove(0);

    assert_eq!(
        project.vitest_setup[0].unresolved_config_extends.as_deref(),
        Some(missing_source.as_str())
    );
    assert!(project.vitest_setup[0].trigger_paths.contains(&missing));
}
