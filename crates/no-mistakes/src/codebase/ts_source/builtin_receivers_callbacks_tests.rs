use super::*;

fn callbacks(source: &str) -> Vec<String> {
    let found = proven_facts(source);
    let mut calls = found
        .callbacks
        .into_iter()
        .filter_map(|((start, _), positions)| {
            positions
                .filter(|positions| !positions.is_empty())
                .map(|_| {
                    source[start as usize..]
                        .split(';')
                        .next()
                        .unwrap()
                        .to_string()
                })
        })
        .collect::<Vec<_>>();
    calls.sort();
    calls
}

fn callback_values(
    source: &str,
    needle: &str,
    facts: &super::super::ProvenCalls,
) -> Vec<Option<Vec<usize>>> {
    let start = source.find(needle).unwrap() as u32;
    facts
        .callbacks
        .iter()
        .filter(|((offset, _), _)| *offset == start)
        .map(|(_, positions)| positions.map(|positions| positions.to_vec()))
        .collect()
}

fn proven_facts(source: &str) -> super::super::ProvenCalls {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, source, SourceType::mjs().with_typescript(true))
        .parse()
        .program;
    collect(&program)
}

fn proof_source(name: &str) -> String {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    std::fs::read_to_string(root.join("src").join(name)).unwrap()
}

#[test]
fn callback_positions_require_the_proven_receiver_family() {
    let source = proof_source("proof-callbacks.mts");
    let facts = proven_facts(&source);
    let found = callbacks(&source);
    assert_eq!(found.len(), 5, "{found:?}");
    assert!(
        found.iter().any(|call| call.contains("Array.from")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call.contains("rows.map")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|call| call.contains("Promise.resolve(1).then")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call.contains("new Store().run")),
        "{found:?}"
    );
    assert_eq!(
        callback_values(&source, "rows.map", &facts),
        vec![Some(vec![0])]
    );
    assert_eq!(
        callback_values(&source, "'text'.replace", &facts),
        vec![Some(vec![1])]
    );
    assert_eq!(
        callback_values(&source, "'text'.replace(/x/", &facts),
        vec![Some(vec![])]
    );
    let promise = callback_values(&source, "Promise.resolve", &facts);
    assert_eq!(promise.len(), 2, "{promise:?}");
    assert!(promise.contains(&None), "{promise:?}");
    assert!(promise.contains(&Some(vec![0, 1])), "{promise:?}");
    let finally_literal = callback_values(&source, "Promise.resolve(1).finally", &facts);
    assert!(finally_literal.contains(&None), "{finally_literal:?}");
    assert!(
        finally_literal.contains(&Some(vec![])),
        "{finally_literal:?}"
    );
    assert_eq!(
        callback_values(&source, "new Store()", &facts),
        vec![Some(vec![1])]
    );
    assert_eq!(
        callback_values(&source, "Array.from", &facts),
        vec![Some(vec![1])]
    );
    assert_eq!(callback_values(&source, "Buffer.from", &facts), vec![None]);
    let leaves = proven(&source);
    assert!(
        leaves.iter().any(|call| call == "empty.forEach"),
        "{leaves:?}"
    );
    assert!(leaves.iter().any(|call| call == "[,,].map"), "{leaves:?}");
    assert_eq!(
        callback_values(&source, "empty.forEach", &facts),
        vec![Some(vec![])]
    );

    let shadowed = proof_source("proof-shadowed-array.mts");
    assert!(proven_facts(&shadowed).callbacks.is_empty());
}

#[test]
fn contextual_primitive_proofs_need_explicit_source_types() {
    let source = proof_source("proof-primitives.mts");
    let found = proven(&source);
    assert_eq!(
        found
            .iter()
            .filter(|call| call.as_str() == "r.toFixed")
            .count(),
        2,
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call == "expectation.function.trim"),
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call == "dayAnchorMs.toString"),
        "{found:?}"
    );
    assert!(
        found.iter().any(|call| call == "dayAnchorText.padStart"),
        "{found:?}"
    );
    assert!(!found.iter().any(|call| call == "r.trim"), "{found:?}");
}

#[test]
fn builtin_provenance_requires_explicit_ownership_demand() {
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let file = root.join("src/proof-callbacks.mts");
    let plain = TsFactPlan {
        function_calls: true,
        value_flow: true,
        ..Default::default()
    };
    let off = collect_ts_facts(std::slice::from_ref(&file), plain);
    let on = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            call_eligibility: true,
            ..plain
        },
    );
    assert!(off.get(&file).unwrap().builtin_callback_calls.is_empty());
    assert!(!on.get(&file).unwrap().builtin_callback_calls.is_empty());
    assert_eq!(
        off.get(&file).unwrap().function_calls,
        on.get(&file).unwrap().function_calls
    );
}

#[test]
fn declared_types_keep_native_families_without_inventing_element_context() {
    let found = proven(&proof_source("proof-types.mts"));
    assert_eq!(
        found,
        [
            "flag.toString",
            "whole.toString",
            "literal.trim",
            "numberLiteral.toFixed",
            "booleanLiteral.valueOf",
            "tuple.join",
            "textual.map",
            "t.trim",
            "mixed.map",
            "arrays.forEach",
            "unknown.forEach",
            "readonlyRows.forEach",
            "n.toFixed",
            "genericRows.forEach",
            "g.trim",
            "genericReadonly.forEach",
            "p.toFixed",
            "nestedReadonly.forEach",
        ]
    );
}
