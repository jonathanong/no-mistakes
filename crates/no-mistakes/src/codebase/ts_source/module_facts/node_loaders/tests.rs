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
fn optional_loader_chains_record_facts_and_label_every_call() {
    use oxc_ast_visit::{walk, Visit};

    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts")
        .join("loader-chains.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let (labels, facts) = crate::ast::with_program(&path, &source, |program, _| {
        struct Labels(Vec<String>);
        impl<'a> Visit<'a> for Labels {
            fn visit_call_expression(&mut self, it: &oxc_ast::ast::CallExpression<'a>) {
                self.0.push(super::label(&it.callee));
                walk::walk_call_expression(self, it);
            }
        }
        let mut labels = Labels(Vec::new());
        labels.visit_program(program);
        (
            labels.0,
            super::super::extract_with_arguments(program, true),
        )
    })
    .unwrap();
    assert_eq!(
        labels,
        vec![
            "module.require".to_string(),
            "module[\"require\"]".to_string(),
            "<Node loader>".to_string(),
            "module.require".to_string(),
            "createRequire(...)".to_string(),
            "createRequire".to_string(),
            "<Node loader>".to_string(),
        ]
    );
    let mut recorded: Vec<_> = facts
        .node_loader_calls
        .values()
        .flatten()
        .map(|call| format!("{} {}", call.callee, call.specifier))
        .collect();
    recorded.sort();
    assert_eq!(
        recorded,
        vec![
            "createRequire(...) fs".to_string(),
            "module.require node:fs".to_string(),
            "module.require node:fs".to_string(),
            "module[\"require\"] node:fs".to_string(),
        ]
    );
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
