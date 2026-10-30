use super::*;

fn read(name: &str, collect: bool) -> (String, TypeScriptModuleFacts) {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan")
        .parent()
        .unwrap()
        .join("node-loaders/tests")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        super::super::extract_with_arguments(program, collect)
    })
    .unwrap();
    (source, facts)
}

#[test]
fn node_loaders_keep_private_potential_provenance_and_exact_offsets() {
    for (file, count) in [
        ("module.test.ts", 6),
        ("factory.test.ts", 10),
        ("commonjs.test.cjs", 3),
        ("long-alias.test.ts", 2),
        ("arguments.test.ts", 1),
    ] {
        let (source, facts) = read(file, true);
        let calls: Vec<_> = facts
            .node_loader_calls
            .iter()
            .flat_map(|(offset, calls)| {
                assert!(source.is_char_boundary(*offset as usize));
                calls
            })
            .collect();
        assert_eq!(calls.len(), count, "{file}: {calls:?}");
        assert!(calls
            .iter()
            .all(|load| !load.callee.contains("<Node loader>")));
        let public = serde_json::to_value(facts).unwrap();
        assert!(public.get("node_loader_calls").is_none());
        assert!(public.get("nodeLoaderCalls").is_none());
    }
}

#[test]
fn loader_facts_require_demand_and_exclude_unrelated_imports_and_shadows() {
    assert!(read("factory.test.ts", false)
        .1
        .node_loader_calls
        .is_empty());
    for name in [
        "shadows.test.ts",
        "type-only.test.ts",
        "cycles.test.ts",
        "unsupported.test.ts",
    ] {
        assert!(read(name, true).1.node_loader_calls.is_empty(), "{name}");
    }
}
