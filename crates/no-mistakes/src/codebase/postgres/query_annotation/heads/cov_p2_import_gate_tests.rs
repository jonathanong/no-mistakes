use super::super::HeadProof;
use crate::codebase::check_facts::{
    tests::compatibility::collect_file_facts_from_program, CheckFactPlan, CheckFileFacts,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::FileIdMap;

fn prepared() -> (FileIdMap<CheckFileFacts>, std::path::PathBuf) {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("cov-p2-import-gate");
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
    let files: FileIdMap<_> = [
        "resolved",
        "unresolved",
        "computed",
        "ungated",
        "flags",
        "producer",
        "records",
        "callbacks",
        "callback-matrix",
        "namespace-matrix",
        "returned-matrix",
    ]
    .into_iter()
    .map(|name| {
        let path = root.join(format!("{name}.mts"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = crate::ast::with_program(&path, &source, |program, source| {
            collect_file_facts_from_program(&root, &path, &plan, None, source, program, None)
        })
        .unwrap();
        (path, facts)
    })
    .collect();
    (files, root)
}

#[test]
fn cov_p2_closed_names_require_resolvable_static_imports_only() {
    let (mut files, root) = prepared();
    let before: FileIdMap<_> = files
        .iter()
        .filter_map(|(path, file)| {
            file.query_annotation_heads
                .clone()
                .map(|heads| (path.clone(), heads))
        })
        .collect();
    super::retain(&mut files, &|specifier, _| match specifier {
        "@bench/db" | "./flags.mjs" => Some(root.join("flags.mts")),
        _ => None,
    });
    for (name, accepted) in [("resolved", true), ("unresolved", false)] {
        let path = root.join(format!("{name}.mts"));
        let original = &before[&path];
        assert_eq!(original.import_gated_finite.len(), 1, "{name}");
        let span = original.import_gated_finite[0];
        assert_eq!(original.calls[&span], HeadProof::Literal);
        let heads = files[&path].query_annotation_heads.as_ref().unwrap();
        assert_eq!(heads.accepts(span), accepted, "{name}");
        for (call, proof) in &original.calls {
            if *call != span {
                assert_eq!(heads.calls[call], *proof, "ordinary SQL in {name}");
            }
        }
    }
    let path = root.join("computed.mts");
    let original = &before[&path];
    let file = &files[&path];
    assert!(file
        .ts
        .module_bindings
        .as_ref()
        .unwrap()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Non-literal dynamic import specifier"));
    assert!(original.import_gated_finite.is_empty());
    let source = std::fs::read_to_string(&path).unwrap();
    let start = source.find("read(").unwrap();
    let span = original
        .calls
        .keys()
        .find(|span| span.0 as usize == start)
        .unwrap();
    assert_eq!(original.calls[span], HeadProof::Unproven);
    let start = source.find("read('/* ordinary").unwrap();
    let span = original
        .calls
        .keys()
        .find(|span| span.0 as usize == start)
        .unwrap();
    assert_eq!(original.calls[span], HeadProof::Literal);
    assert_eq!(
        file.query_annotation_heads.as_ref().unwrap().calls,
        original.calls
    );
    let path = root.join("ungated.mts");
    let original = &before[&path];
    assert!(original.import_gated_finite.is_empty());
    assert_eq!(
        files[&path].query_annotation_heads.as_ref().unwrap().calls,
        original.calls
    );
}

#[test]
fn cov_p2_destructuring_requires_plain_record_source_not_builder() {
    let (mut files, root) = prepared();
    let path = root.join("records.mts");
    let heads = files[&path].query_annotation_heads.as_ref().unwrap();
    assert_eq!(
        heads
            .pending_imports
            .iter()
            .filter(|pending| matches!(pending.kind, super::super::helper_heads::Kind::PlainRecord))
            .count(),
        2
    );
    super::super::import_projection::project(&mut files, |specifier, _| {
        (specifier == "./producer.mjs").then(|| root.join("producer.mts"))
    });
    let heads = files[&path].query_annotation_heads.as_ref().unwrap();
    let source = std::fs::read_to_string(&path).unwrap();
    let accepted = source
        .match_indices("read(")
        .map(|(start, _)| {
            let span = heads
                .calls
                .keys()
                .find(|span| span.0 as usize == start)
                .unwrap();
            heads.accepts(*span)
        })
        .collect::<Vec<_>>();
    assert_eq!(accepted, [true, false]);
}

#[path = "cov_p2_projection_tests.rs"]
mod projection_tests;
