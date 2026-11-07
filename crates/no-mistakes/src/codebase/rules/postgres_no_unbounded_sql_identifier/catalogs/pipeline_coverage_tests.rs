use super::super::test_support::{binding, prepared};
use super::*;

fn call<'a>(flow: &'a ValueFlowFacts, value: &ValueExpression) -> &'a ValueCall {
    let ValueExpression::Call(start, end) = value else {
        panic!("expected call")
    };
    flow.calls
        .iter()
        .find(|call| (call.offset, call.end) == (*start, *end))
        .unwrap()
}

fn receiver<'a>(flow: &'a ValueFlowFacts, value: &ValueExpression) -> &'a ValueCall {
    let outer = call(flow, value);
    let ValueExpression::Member(base, _) = &outer.callee else {
        panic!("expected method")
    };
    call(flow, base)
}

#[test]
fn catalog_pipeline_variants_preserve_conservative_callback_and_list_boundaries() {
    let (facts, graph, root) = prepared(&["pipelines.ts"]);
    let file = root.join("pipelines.ts");
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let proof = CatalogProof::new(&graph, &facts, &bindings);
    for (name, expected) in [
        ("GOOD", true),
        ("INVALID_ARGUMENTS", false),
        ("NON_MEMBER", false),
        ("UNSUPPORTED", false),
        ("NAMED_CALLBACK", false),
        ("BLOCK_CALLBACK", false),
        ("MISSING_FIELD", false),
        ("FILTER_JOIN", false),
        ("UNKNOWN_CALLBACK", false),
        ("BOUND", false),
        ("NO_ARGUMENTS", false),
        ("MUTABLE", false),
        ("CYCLIC", false),
        ("ALREADY_STRINGS", false),
    ] {
        assert_eq!(
            proof.pipeline(&file, &binding(&facts, &file, name).value),
            expected,
            "{name}"
        );
    }
}

#[test]
fn catalog_collection_consumers_require_known_pure_callbacks_and_literal_joins() {
    let (facts, graph, root) = prepared(&["pipelines.ts"]);
    let file = root.join("pipelines.ts");
    let index = index::Index::new(&graph, &facts);
    let source = *index.sources.get(&(file.as_path(), 0, "BAD_ROWS")).unwrap();
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    for (name, expected) in [
        ("INVALID_ARGUMENTS", false),
        ("NON_MEMBER", false),
        ("UNSUPPORTED", false),
        ("NAMED_CALLBACK", false),
        ("BLOCK_CALLBACK", false),
        ("MISSING_FIELD", false),
        ("FILTER_JOIN", true),
        ("UNKNOWN_CALLBACK", false),
        ("BOUND", false),
        ("NO_ARGUMENTS", false),
    ] {
        let inner = receiver(flow, &binding(&facts, &file, name).value);
        assert_eq!(
            collection_safe(&index, &file, source, inner.offset, inner.end),
            expected,
            "{name}"
        );
    }
    for (name, expected) in [("LITERAL_JOIN", true), ("OPAQUE_CALL", false)] {
        let call = call(flow, &binding(&facts, &file, name).value);
        assert_eq!(
            collection_safe(&index, &file, source, call.offset, call.end),
            expected,
            "{name}"
        );
    }
}

#[test]
fn catalog_pipeline_proofs_stop_when_retained_call_or_callback_context_is_missing() {
    let (facts, graph, root) = prepared(&["pipelines.ts"]);
    let file = root.join("pipelines.ts");
    let bindings = Bindings::new(&facts).with_graph(&graph);
    let mut proof = CatalogProof::new(&graph, &facts, &bindings);
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let value = &binding(&facts, &file, "GOOD").value;
    assert!(proof.pipeline(&file, value));
    let outer = call(flow, value);
    proof
        .index
        .calls
        .remove(&(file.as_path(), outer.offset, outer.end));
    assert!(!proof.pipeline(&file, value));
    let source = *proof
        .index
        .sources
        .get(&(file.as_path(), 0, "BAD_ROWS"))
        .unwrap();
    assert!(!collection_safe(
        &proof.index,
        &file,
        source,
        outer.offset,
        outer.end
    ));
    let inner = receiver(flow, &binding(&facts, &file, "MISSING_FIELD").value);
    proof.index.flows.remove(file.as_path());
    assert!(!collection_safe(
        &proof.index,
        &file,
        source,
        inner.offset,
        inner.end
    ));
}
