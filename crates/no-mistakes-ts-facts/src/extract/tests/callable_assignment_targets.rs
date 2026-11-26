use super::*;

#[test]
fn member_writes_skip_callable_assignment_target_construction() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases");
    for relative in [
        "codebase-analysis/value-flow-replacement-scale-16/fixture/src/entry.mts",
        "codebase-analysis/value-flow-foreign-parameter-timing/fixture/src/entry.mts",
    ] {
        let source = std::fs::read_to_string(root.join(relative)).unwrap();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty());
        for values in [false, true] {
            let facts = extract_import_facts_from_program_with_source_and_demands(
                &parsed.program,
                &source,
                false,
                false,
                values,
            );
            // These physical receiver writes cannot assign a lexical module binding.
            #[cfg(feature = "test-instrumentation")]
            assert_eq!(
                facts.value_flow.callable_assignment_target_constructions, 0,
                "{relative}/{values}"
            );
            assert!(facts.value_flow.callable_assignments().is_empty());
            if values {
                assert!(!facts
                    .value_flow
                    .replacements
                    .as_ref()
                    .unwrap()
                    .writes
                    .is_empty());
            }
        }
    }
    let facts = super::saved_assignment_facts("mixed-callable-receiver.mts", true);
    #[cfg(feature = "test-instrumentation")]
    assert_eq!(facts.value_flow.callable_assignment_target_constructions, 1);
    assert_eq!(facts.value_flow.callable_assignments().len(), 1);
    assert!(facts.value_flow.callable_assignment_is_used(0, "callback"));
}

#[test]
fn callable_staging_rejects_missing_shadowed_and_imported_binding_scopes() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/mixed-callable-receiver.mts",
    );
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let assignment = parsed
        .program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::ExpressionStatement(statement) = statement else {
                return None;
            };
            let Expression::AssignmentExpression(assignment) = &statement.expression else {
                return None;
            };
            Some(assignment)
        })
        .unwrap();
    // Exercise conservative staging boundaries with the same saved physical
    // assignment: no visible binding, an inner shadow, and an import identity.
    for boundary in ["missing", "shadowed", "imported"] {
        let mut collector = ImportCollector {
            collect_value_flow: true,
            ..Default::default()
        };
        collector
            .callable_assignment_state
            .get_or_insert_with(Default::default)
            .module_bindings
            .insert("callback".to_string());
        if boundary != "missing" {
            collector
                .local_stack
                .push(["callback".to_string()].into_iter().collect());
            collector.lexical_scope_ids.push(0);
        }
        if boundary == "shadowed" {
            collector
                .local_stack
                .push(["callback".to_string()].into_iter().collect());
            collector.lexical_scope_ids.push(1);
        }
        if boundary == "imported" {
            collector.imported_bindings.insert("callback".to_string());
        }
        collector.record_callable_assignment(
            &assignment.left,
            Some(&assignment.right),
            true,
            assignment.span.end,
        );
        #[cfg(feature = "test-instrumentation")]
        assert_eq!(
            collector
                .value_flow
                .callable_assignment_target_constructions,
            1
        );
        assert!(
            collector
                .callable_assignment_state
                .unwrap()
                .assignments
                .is_empty(),
            "{boundary}"
        );
    }
}
