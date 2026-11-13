use crate::codebase::check_facts::{
    sql_source_prerequisites::{SqlProjection, SqlSourceKey},
    tests::compatibility::collect_file_facts_from_program,
    CheckFactPlan, CheckFileFacts,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::FileIdMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn prepared() -> (FileIdMap<CheckFileFacts>, PathBuf) {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("coverage-head-projection");
    let profiles = vec![EmbeddedSqlOptions::configured(
        "@bench/db",
        &["read".into()],
    )];
    let mut plan = CheckFactPlan {
        query_annotation_heads: true,
        query_annotation_options: Some(profiles.clone()),
        embedded_sql_options: profiles,
        ..Default::default()
    };
    plan.graph.imports = true;
    let names: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    let files = names
        .into_iter()
        .map(|name| {
            let path = root.join(name);
            let source = std::fs::read_to_string(&path).unwrap();
            let parsed = crate::ast::with_program(&path, &source, |program, source| {
                collect_file_facts_from_program(&root, &path, &plan, None, source, program, None)
            });
            let facts = parsed.unwrap_or_else(|error| {
                assert!(
                    matches!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some("broken.mts" | "duplicate.mts")
                    ),
                    "unexpected fixture parse failure in {}: {error}",
                    path.display()
                );
                CheckFileFacts {
                    parse_error: Some(error.to_string()),
                    ..Default::default()
                }
            });
            (path, facts)
        })
        .collect();
    (files, root)
}

fn resolve(files: &FileIdMap<CheckFileFacts>) -> impl Fn(&str, &Path) -> Option<PathBuf> {
    let paths = files.keys().cloned().collect::<BTreeSet<_>>();
    move |specifier, from| {
        let target = from.parent()?.join(specifier).with_extension("mts");
        paths.get(&target).cloned()
    }
}

fn assert_projected(files: &FileIdMap<CheckFileFacts>, root: &Path, name: &str, count: usize) {
    let path = root.join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let file = &files[&path];
    let heads = file.query_annotation_heads.as_ref().unwrap();
    let key = SqlSourceKey::profile(SqlProjection::AnnotationHeads, &file.embedded_sql[0].0);
    let failures: BTreeSet<_> = file
        .sql_source_prerequisites
        .failures(&key)
        .cloned()
        .collect();
    let expected = if name == "callback-callers.mts" {
        BTreeSet::new()
    } else {
        BTreeSet::from([root.join("broken.mts")])
    };
    assert_eq!(
        failures, expected,
        "{name}: only unreadable source dependencies are advisories"
    );
    let mut offset = 0;
    let mut checked = 0;
    let mut mismatches = Vec::new();
    for line in source.split_inclusive('\n') {
        if let Some((statement, marker)) = line.split_once("// low: ") {
            let start = offset + statement.find("read(").unwrap();
            let expected: bool = marker.trim().parse().unwrap();
            let actual = heads
                .calls
                .keys()
                .find(|span| span.0 as usize == start)
                .map(|span| heads.accepts(*span));
            if actual != Some(expected) {
                mismatches.push(format!(
                    "{name}: {}: expected {expected}, got {actual:?}",
                    statement.trim()
                ));
            }
            checked += 1;
        }
        offset += line.len();
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert_eq!(checked, count);
}

#[test]
fn namespace_projection_follows_runtime_aliases_and_bounds_reexport_walks() {
    let (mut files, root) = prepared();
    let caller = root.join("namespace.mts");
    let pending = &files[&caller]
        .query_annotation_heads
        .as_ref()
        .unwrap()
        .pending_namespaces;
    assert_eq!(
        pending.len(),
        13,
        "escaped namespace must not request a source proof"
    );
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    assert_projected(&files, &root, "namespace.mts", 14);
}

#[test]
fn direct_imports_preserve_value_record_and_builder_result_contracts() {
    let (mut files, root) = prepared();
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    assert_projected(&files, &root, "direct.mts", 15);
}

#[test]
fn async_factories_require_every_second_hop_append_effect() {
    let (mut files, root) = prepared();
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    assert_projected(&files, &root, "async.mts", 9);
}

#[test]
fn callback_payloads_only_cross_exact_stable_append_helper_boundaries() {
    let (mut files, root) = prepared();
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    assert_projected(&files, &root, "callback-callers.mts", 16);
}

#[test]
fn returned_parameter_projection_requires_literal_suffixes_and_every_helper_arm() {
    let (mut files, root) = prepared();
    let resolver = resolve(&files);
    super::super::project(&mut files, resolver);
    assert_projected(&files, &root, "returned.mts", 17);
}
