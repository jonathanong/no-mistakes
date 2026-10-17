use super::*;
use crate::codebase::postgres::TrustedSqlTag;
use crate::codebase::ts_resolver::{load_tsconfig, ImportResolver};

const FIXTURE: &str = "helper-tracing-resolved-external-roots";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture")
        .join(FIXTURE)
        .canonicalize()
        .unwrap()
}

#[test]
fn resolved_external_edges_do_not_hide_workspace_reexports_or_uncertain_imports() {
    let mut files = prepared_fixture(
        FIXTURE,
        &[
            "src/external-only",
            "src/workspace-entry",
            "src/external-then-workspace",
            "src/dynamic",
            "src/unresolved",
            "src/executor-source",
            "src/trusted-source",
            "src/type-only",
            "packages/builders/index",
            "packages/executor",
        ],
    );
    for (_, file) in &mut files {
        file.query_annotation.truncate(1);
        file.embedded_sql.truncate(1);
    }
    let root = root();
    let config = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = ImportResolver::new(&config);
    assert_eq!(
        resolver.resolve("vendor/value", &root.join("src/external-only.mts")),
        Some(root.join("vendor/value.json")),
    );
    let collected = files
        .iter()
        .map(|(path, file)| {
            (
                path.clone(),
                super::super::File {
                    module: Arc::new(super::super::Module::new(Arc::clone(&file.ts))),
                    facts: &file.query_annotation[0].1,
                    executors: file.embedded_sql[0].1.call_spans.iter().copied().collect(),
                },
            )
        })
        .collect();
    let options =
        EmbeddedSqlOptions::configured("@app/db", &[]).with_trusted_sql_tags(&[TrustedSqlTag {
            module: "@app/tags".into(),
            name: "tag".into(),
        }]);
    let selected = super::super::demand::roots(&collected, &options, &|specifier, from| {
        resolver.resolve(specifier, from)
    })
    .into_iter()
    .map(|path| path.strip_prefix(&root).unwrap().to_path_buf())
    .collect::<BTreeSet<_>>();
    assert_eq!(
        selected,
        BTreeSet::from([
            PathBuf::from("src/workspace-entry.mts"),
            PathBuf::from("src/external-then-workspace.mts"),
            PathBuf::from("src/dynamic.mts"),
            PathBuf::from("src/unresolved.mts"),
            PathBuf::from("src/executor-source.mts"),
            PathBuf::from("src/trusted-source.mts"),
            PathBuf::from("packages/builders/index.mts"),
            PathBuf::from("packages/executor.mts"),
        ])
    );
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn public_check_prunes_external_only_root_and_preserves_the_workspace_finding() {
    let (output, diagnostics) =
        crate::benchmark_support::check_json_observed(&root(), true).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["rules"].as_array().unwrap().len(), 1);
    assert_eq!(report["rules"][0]["file"], "packages/executor.mts");
    assert_eq!(report["rules"][0]["line"], 2);
    // The previous implementation keeps nine roots: successful external
    // resolution and a missing target were both treated as uncertainty.
    assert_eq!(diagnostics.work["postgres.annotation.modules"], 10);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 8);
}

#[cfg(all(unix, feature = "test-instrumentation"))]
#[test]
fn canonical_workspace_alias_without_exact_inventory_identity_stays_conservative() {
    let root = root().with_file_name("helper-tracing-resolved-external-alias");
    let config = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = ImportResolver::new(&config);
    let resolved = resolver
        .resolve("@workspace/shared", &root.join("entry.mts"))
        .unwrap();
    assert_eq!(
        resolved,
        root.with_file_name("helper-tracing-resolved-external-workspace")
            .join("shared.mts")
    );
    assert_eq!(root.join("linked.mts").canonicalize().unwrap(), resolved);
    let (output, diagnostics) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["rules"].as_array().unwrap().len(), 1);
    assert_eq!(report["rules"][0]["file"], "linked.mts");
    // Resolver canonical membership alone does not identify a catalog source.
    // The external-looking workspace edge must retain its importing root.
    assert_eq!(diagnostics.work["postgres.annotation.modules"], 2);
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 2);
}
