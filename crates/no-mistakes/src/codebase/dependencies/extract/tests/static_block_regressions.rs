use super::*;

#[test]
fn call_facts_do_not_hoist_class_static_block_vars_into_the_module() {
    let source = r#"
        setTimeout();
        class Timers {
          static { var setTimeout = () => {}; }
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "setTimeout" && call.target_identity == CallTargetIdentity::Global
    }));
}

#[test]
fn call_facts_keep_imported_bindings_outside_class_static_block_shadows() {
    let source = r#"
        import { target } from "./target.mts";
        class Service {
          static {
            const target = () => {};
            target();
          }
        }
        target();
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let calls = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "target")
        .collect::<Vec<_>>();

    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[0].target_identity,
        CallTargetIdentity::RepositoryFunction
    );
    assert_eq!(calls[1].target_identity, CallTargetIdentity::ModuleExport);
}
