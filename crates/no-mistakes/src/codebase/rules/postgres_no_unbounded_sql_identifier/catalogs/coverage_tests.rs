use super::test_support::{binding, helper, prepared};
use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;

#[test]
fn catalog_helpers_reject_async_unused_spread_written_and_namespace_callers() {
    let (facts, graph, root) = prepared(&["helpers.ts", "namespace-caller.ts"]);
    let file = root.join("helpers.ts");
    let index = index::Index::new(&graph, &facts);
    for (name, source, expected) in [
        ("getGood", "CLOSED", true),
        ("getAsync", "ASYNC", false),
        ("nonAccessor", "CLOSED", false),
        ("getUnused", "UNUSED", false),
        ("getRaw", "RAW", false),
        ("getSpread", "SPREAD", false),
        ("getWritten", "WRITTEN", false),
        ("getNamespace", "NAMESPACE", false),
        ("getAlternative", "ALTERNATIVE", false),
    ] {
        let id = helper(&facts, &file, name);
        assert_eq!(
            helpers::safe(&index, (file.as_path(), id), (file.as_path(), 0, source)),
            expected,
            "{name}"
        );
    }
    let alternative = (file.as_path(), helper(&facts, &file, "getAlternative"));
    assert!(index.bad_helpers.contains(&alternative));
}

#[test]
fn catalog_helper_proofs_require_complete_unambiguous_prepared_summaries() {
    let (facts, graph, root) = prepared(&["helpers.ts", "namespace-caller.ts"]);
    let file = root.join("helpers.ts");
    let id = helper(&facts, &file, "getGood");
    let helper = (file.as_path(), id);
    let source = (file.as_path(), 0, "CLOSED");
    let mut index = index::Index::new(&graph, &facts);
    assert!(helpers::safe(&index, helper, source));
    let flow = index.flows.remove(file.as_path()).unwrap();
    assert!(!helpers::safe(&index, helper, source));
    index.flows.insert(file.as_path(), flow);
    let returns = index.returns.remove(&helper).unwrap();
    assert!(!helpers::safe(&index, helper, source));
    index.returns.insert(helper, returns);
    let parameters = index.parameters.remove(&helper).unwrap();
    assert!(!helpers::safe(&index, helper, source));
    index.parameters.insert(helper, parameters);
    let catalog = index.sources.remove(&source).unwrap();
    assert!(!helpers::safe(&index, helper, source));
    index.sources.insert(source, catalog);
    let key = index.incoming[&helper][0];
    let call = index.calls.remove(&key).unwrap();
    assert!(!helpers::safe(&index, helper, source));
    index.calls.insert(key, call);
    let original = index.results[&key][0];
    index.results.get_mut(&key).unwrap().push(original);
    assert!(!helpers::safe(&index, helper, source));
    index.results.get_mut(&key).unwrap().pop();
    assert!(helpers::safe(&index, helper, source));
}

#[test]
fn catalog_source_proofs_reject_missing_or_multiple_binding_identities() {
    let (facts, graph, root) = prepared(&["pipelines.ts"]);
    let file = root.join("pipelines.ts");
    let source = (file.as_path(), 0, "ROWS");
    let mut index = index::Index::new(&graph, &facts);
    let catalog = *index.sources.get(&source).unwrap();
    assert!(index.source_safe(source, catalog));
    assert!(!index.source_safe((file.as_path(), 0, "unregistered"), catalog));
    let original = index.bindings[&source][0];
    index.bindings.get_mut(&source).unwrap().push(original);
    assert!(!index.source_safe(source, catalog));
    index.bindings.get_mut(&source).unwrap().pop();
    assert!(index.source_safe(source, catalog));
}

#[test]
fn catalog_rule_proof_rejects_mutable_result_bindings_before_traversal() {
    let (facts, graph, root) = prepared(&["pipelines.ts"]);
    let file = root.join("pipelines.ts");
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let proof = CatalogProof::new(&graph, &facts, &bindings);
    let binding = binding(&facts, &file, "mutableRows");
    assert!(!binding.immutable);
    let embedded = EmbeddedSqlCall::default();
    let call = SqlCall {
        file: file.clone(),
        span: (0, u32::MAX),
        call: &embedded,
    };
    let source = Provenance::Reference {
        name: binding.name.clone(),
        scope: binding.scope,
        offset: u32::MAX,
        members: Vec::new(),
    };
    assert!(!proof.proven(&call, &source));
}

#[test]
fn escaped_catalog_namespaces_stop_at_visited_import_cycles() {
    let (facts, graph, root) = prepared(&["namespace-cycle-a.ts", "namespace-cycle-b.ts"]);
    let file = root.join("namespace-cycle-a.ts");
    let index = index::Index::new(&graph, &facts);
    let escaped = namespace_escapes::collect(&index, &graph);
    assert!(escaped.contains(&(file.as_path(), helper(&facts, &file, "getCycle"))));
}
