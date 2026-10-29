use super::{visit, Pending, Span};
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::Program;

fn pending(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    executors: &FxHashSet<Span>,
) -> Vec<Pending> {
    visit(program, module, executors, &FxHashMap::default(), None).0
}

#[test]
fn caller_projection_requires_an_exact_inline_executor_and_trusted_payload() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-heads/src/query-caller-shapes.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let facts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                imports: true,
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let module = facts.module_bindings.as_ref().unwrap();
        assert!(module.diagnostics.is_empty());
        let calls = crate::codebase::postgres::extract_embedded_sql_from_program(
            &path,
            program,
            &source,
            &crate::codebase::postgres::EmbeddedSqlOptions::configured("@bench/db", &[]),
        );
        assert_eq!(calls.call_spans.len(), 6);
        let projected = pending(program, module, &calls.call_spans.into_iter().collect());
        // The source-visible callback proof also accepts a block body and one
        // ordinary options object; the untrusted member-tag payload stays false.
        assert_eq!(projected.len(), 4);
        assert_eq!(projected[0].payloads, [true, false]);
        assert_eq!(projected[1].payloads, [true, false]);
        assert_eq!(projected[2].payloads, [true, false]);
        assert_eq!(projected[3].payloads, [false, false]);
    })
    .unwrap();
}

#[test]
fn callback_import_indexes_are_demanded_once_after_many_ordinary_calls() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("callback-heads/src/query-many.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let facts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                imports: true,
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let module = facts.module_bindings.as_ref().unwrap();
        assert!(module.diagnostics.is_empty());
        let marker = "read(statement)";
        let executors = FxHashSet::from_iter(source.match_indices(marker).map(|(start, _)| {
            let start = start as u32;
            (start, start + marker.len() as u32)
        }));
        assert_eq!(executors.len(), 2);

        let idle = crate::diagnostics::InvocationObserver::new(true);
        let empty = crate::diagnostics::with_observer(Some(idle.clone()), || {
            pending(program, module, &FxHashSet::default())
        });
        assert!(empty.is_empty());
        assert!(!idle
            .snapshot()
            .work
            .contains_key("postgres.annotation.callback_import_indexes"));
        assert!(!idle
            .snapshot()
            .work
            .contains_key("postgres.annotation.callback_local_reference_visits"));

        let active = crate::diagnostics::InvocationObserver::new(true);
        let found = crate::diagnostics::with_observer(Some(active.clone()), || {
            pending(program, module, &executors)
        });
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].specifier, "./helpers.mts");
        assert_eq!(found[0].imported, "page");
        assert!(found
            .iter()
            .all(|pending| pending.payloads == [true, false]));
        let work = active.snapshot().work;
        assert_eq!(work["postgres.annotation.callback_import_indexes"], 1);
        let references = module
            .bindings
            .iter()
            .filter(|binding| binding.imported && binding.runtime)
            .flat_map(|binding| {
                binding
                    .references
                    .iter()
                    .filter(|reference| reference.runtime)
            })
            .count();
        assert_eq!(
            work["postgres.annotation.callback_import_reference_visits"], references as u64,
            "reference scan runs once across two callbacks and twelve ordinary imported calls"
        );
        let roots = module
            .scopes
            .iter()
            .filter(|scope| scope.parent_id.is_none())
            .map(|scope| scope.id)
            .collect::<FxHashSet<_>>();
        let local_references = module
            .bindings
            .iter()
            .filter(|binding| {
                binding.runtime && !binding.imported && roots.contains(&binding.scope_id)
            })
            .flat_map(|binding| binding.references.iter().filter(|usage| usage.runtime))
            .count();
        assert_eq!(
            work["postgres.annotation.callback_local_reference_visits"],
            local_references as u64
        );

        let callee = source.rfind("page(sql").unwrap() as u32;
        let callee = (callee, callee + 4);
        let owner = module
            .bindings
            .iter()
            .find(|binding| {
                binding
                    .references
                    .iter()
                    .any(|reference| (reference.span.start, reference.span.end) == callee)
            })
            .unwrap();
        let mut duplicate_owner = module.as_ref().clone();
        let mut another = owner.clone();
        another.id = duplicate_owner
            .bindings
            .iter()
            .map(|binding| binding.id)
            .max()
            .unwrap()
            + 1;
        duplicate_owner.bindings.push(another);
        assert!(pending(program, &duplicate_owner, &executors).is_empty());

        let mut duplicate_import = module.as_ref().clone();
        let import = duplicate_import
            .imports
            .iter_mut()
            .find(|import| {
                import
                    .bindings
                    .iter()
                    .any(|binding| binding.binding_id == owner.id)
            })
            .unwrap();
        let binding = import
            .bindings
            .iter()
            .find(|binding| binding.binding_id == owner.id)
            .unwrap()
            .clone();
        import.bindings.push(binding);
        assert!(pending(program, &duplicate_import, &executors).is_empty());

        let mut type_only = module.as_ref().clone();
        let import = type_only
            .imports
            .iter_mut()
            .find(|import| {
                import
                    .bindings
                    .iter()
                    .any(|binding| binding.binding_id == owner.id)
            })
            .unwrap();
        import
            .bindings
            .iter_mut()
            .find(|binding| binding.binding_id == owner.id)
            .unwrap()
            .type_only = true;
        assert!(pending(program, &type_only, &executors).is_empty());

        let mut namespace = module.as_ref().clone();
        let import = namespace
            .imports
            .iter_mut()
            .find(|import| {
                import
                    .bindings
                    .iter()
                    .any(|binding| binding.binding_id == owner.id)
            })
            .unwrap();
        import
            .bindings
            .iter_mut()
            .find(|binding| binding.binding_id == owner.id)
            .unwrap()
            .imported = "*".into();
        assert!(pending(program, &namespace, &executors).is_empty());

        let mut ambiguous_tag = module.as_ref().clone();
        let tag = ambiguous_tag
            .imports
            .iter_mut()
            .find(|import| import.specifier == "sql-template-strings")
            .unwrap();
        tag.bindings.push(tag.bindings[0].clone());
        let pending = pending(program, &ambiguous_tag, &executors);
        assert_eq!(pending.len(), 2);
        assert!(pending
            .iter()
            .all(|pending| pending.payloads == [false, false]));
    })
    .unwrap();
}
