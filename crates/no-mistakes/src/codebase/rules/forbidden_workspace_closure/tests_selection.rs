use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::schema::{RuleDef, RuleScope};
use std::sync::Arc;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-workspace-closure")
            .join(name),
    )
}

fn observed_sources(
    files: &[PathBuf],
) -> (SourceStore, Arc<crate::diagnostics::InvocationObserver>) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(files)),
        Some(Arc::clone(&observer)),
    );
    (sources, observer)
}

#[test]
fn layered_workspace_reconstructs_only_the_reported_chain() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/performance/workspace-closure-paths"),
    );
    let mut files = vec![root.join("package.json")];
    for layer in 0..32 {
        for side in ['a', 'b'] {
            files.push(root.join(format!("packages/p{layer:02}{side}/package.json")));
        }
    }
    files.push(root.join("packages/app/package.json"));
    let mut config = NoMistakesConfig::default();
    config.rules.push(application(
        "packages: ['@perf/app']\nforbidden: [fixture-denied]\n",
    ));
    let (sources, observer) = observed_sources(&files);

    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();

    assert_eq!(findings.len(), 1);
    let expected_chain = std::iter::once("@perf/app".to_string())
        .chain((0..32).map(|layer| format!("@perf/p{layer:02}a")))
        .chain(std::iter::once("fixture-denied".to_string()))
        .collect::<Vec<_>>()
        .join(" -> ");
    assert_eq!(findings[0].import.as_deref(), Some(expected_chain.as_str()));
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.path_reconstructions"],
        1,
        "the traversal should materialize a path only for the single emitted finding"
    );
}

fn application(options: &str) -> RuleDef {
    RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    }
}

#[test]
fn saved_dependency_benchmark_shares_one_workspace_and_all_field_projection() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/performance/dependency-checks"),
    );
    let mut files = vec![root.join("package.json")];
    files.extend((0..24).map(|package| root.join(format!("packages/p{package:02}/package.json"))));
    let mut config = NoMistakesConfig::default();
    for package in ["@perf/p00", "@perf/p08", "@perf/p16"] {
        config.rules.push(application(&format!(
            "packages: ['{package}']\nforbidden: [fixture-denied]\nlockfile: pnpm-lock.yaml\n"
        )));
    }
    let (sources, observer) = observed_sources(&files);
    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert!(findings.is_empty());
    let work = observer.snapshot().work;
    // Output equality would also pass with six discarded/repeated projections.
    assert_eq!(work["forbidden_workspace_closure.workspace_selections"], 1);
    assert_eq!(work["forbidden_workspace_closure.manifest_projections"], 1);
    assert_eq!(
        work["forbidden_workspace_closure.lockfile_importer_constructions"],
        1
    );
}

#[test]
fn shared_lockfile_projection_keeps_selected_fields_and_manifest_peers() {
    let root = fixture_root("lockfile-dependency-types");
    let files = vec![
        root.join("package.json"),
        root.join("packages/app/package.json"),
    ];
    let mut config = NoMistakesConfig::default();
    for field in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        config.rules.push(application(&format!(
            "packages: ['@acme/app']\nforbidden: ['@acme/secret-*']\ndependencyTypes: [{field}]\nlockfile: pnpm-lock.yaml\n"
        )));
    }
    let (sources, observer) = observed_sources(&files);
    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    let targets: BTreeSet<_> = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    assert_eq!(
        targets,
        BTreeSet::from([
            "@acme/secret-dev",
            "@acme/secret-optional",
            "@acme/secret-peer"
        ])
    );
    assert_eq!(findings.len(), 3);
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.manifest_projections"],
        1
    );
}

#[test]
fn manifest_applications_share_only_equivalent_field_selections() {
    let root = fixture_root("lockfile-dependency-types");
    let files = vec![
        root.join("package.json"),
        root.join("packages/app/package.json"),
    ];
    let mut config = NoMistakesConfig::default();
    for field in ["dependencies", "peerDependencies", "peerDependencies"] {
        config.rules.push(application(&format!(
            "packages: ['@acme/app']\nforbidden: ['@acme/secret-peer']\ndependencyTypes: [{field}]\n"
        )));
    }
    let (sources, observer) = observed_sources(&files);
    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("@acme/secret-peer")));
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.workspace_selections"],
        1
    );
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.manifest_projections"],
        2
    );
}

#[test]
fn external_root_selection_and_duplicate_name_precedence_remain_distinct() {
    let fixture = fixture_root("selection-roots");
    let root = fixture.join("repository");
    let first = fixture.join("first");
    let second = fixture.join("second");
    let files = vec![
        root.join("package.json"),
        first.join("package.json"),
        second.join("package.json"),
    ];
    let (sources, observer) = observed_sources(&files);
    let cache = selection::SelectionCache::default();
    let first_roots = vec![first.clone(), second.clone()];
    let second_roots = vec![second, first];
    let a = cache
        .get_or_compute(&root, &first_roots, &files, &sources)
        .unwrap();
    let repeated = cache
        .get_or_compute(&root, &first_roots, &files, &sources)
        .unwrap();
    let b = cache
        .get_or_compute(&root, &second_roots, &files, &sources)
        .unwrap();
    assert!(Arc::ptr_eq(&a, &repeated));
    assert!(!Arc::ptr_eq(&a, &b));
    let a_nodes = a.manifest_nodes(&["dependencies"], &sources);
    let b_nodes = b.manifest_nodes(&["dependencies"], &sources);
    assert_eq!(a_nodes["@scope/shared"].deps[0].name, "second-only");
    assert_eq!(b_nodes["@scope/shared"].deps[0].name, "first-only");
    assert_eq!(
        observer.snapshot().work["forbidden_workspace_closure.workspace_selections"],
        2
    );
}
