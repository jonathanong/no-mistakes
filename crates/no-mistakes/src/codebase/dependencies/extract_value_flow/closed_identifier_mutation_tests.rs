use super::super::super::super::ImportCollector;
use crate::codebase::dependencies::extract::{
    CallableId, ImportedBinding, ImportedBindingKind, ValueBinding, ValueExpression, ValueParameter,
};
use crate::fx::fx_set;
use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, Program, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

const SOURCE: &str = "\
const plain = marker;
crypto.randomUUID();
tables.set(1);
";

#[test]
fn closed_registry_calls_skip_when_value_flow_is_off() {
    let allocator = Allocator::default();
    let parsed = parse(&allocator);
    let call = call_with_receiver(&parsed.program, "tables");
    let mut collector = ImportCollector::default();
    collector.note_closed_registry_call(call);
    assert!(collector.value_flow.registry_mutations.is_empty());
}

#[test]
fn uuid_trust_requires_a_crypto_import_without_a_module_local() {
    let allocator = Allocator::default();
    let parsed = parse(&allocator);
    let program = &parsed.program;
    let mut collector = module_collector();
    assert!(!collector.trusted_uuid_call(plain_init(program)));

    let crypto = expression_with_receiver(program, "crypto");
    assert!(trust_import(
        ImportedBindingKind::Default,
        "node:crypto",
        crypto
    ));
    assert!(trust_import(
        ImportedBindingKind::Namespace,
        "crypto",
        crypto
    ));
    assert!(!trust_import(
        ImportedBindingKind::Named,
        "node:crypto",
        crypto
    ));

    collector.value_flow.bindings.push(other_binding());
    collector.value_flow.parameters.push(other_parameter());
    collector.record_callable_binding_id("crypto", CallableId(1));
    assert!(!collector.trusted_uuid_call(crypto));
}

fn trust_import(kind: ImportedBindingKind, specifier: &str, call: &Expression<'_>) -> bool {
    let mut collector = module_collector();
    collector.call_import_bindings.push(ImportedBinding {
        specifier: specifier.to_string(),
        local: "crypto".to_string(),
        imported: match kind {
            ImportedBindingKind::Default => "default",
            ImportedBindingKind::Namespace => "*",
            ImportedBindingKind::Named => "crypto",
        }
        .to_string(),
        kind,
        is_type_only: false,
    });
    collector.trusted_uuid_call(call)
}

fn module_collector() -> ImportCollector {
    let mut collector = ImportCollector::default();
    collector.local_stack.push(fx_set());
    collector.lexical_scope_ids.push(0);
    collector.lexical_scope_parents.insert(0, None);
    collector.next_lexical_scope_id = 1;
    collector.add_binding_name("crypto");
    collector
}

fn other_binding() -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: "other".to_string(),
        scope: 0,
        declared_at: 1,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        value: ValueExpression::Unknown,
    }
}

fn other_parameter() -> ValueParameter {
    ValueParameter {
        owner: CallableId(2),
        index: 0,
        name: "other".to_string(),
        scope: 0,
        default: ValueExpression::Unknown,
        projection: Vec::new(),
        initialization_order: 0,
        default_order: 0,
        default_mutations: None,
        invalidated_at: None,
        mutation_contexts: fx_set(),
        type_reference: None,
    }
}

fn parse(allocator: &Allocator) -> oxc_parser::ParserReturn<'_> {
    let parsed = Parser::new(allocator, SOURCE, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    parsed
}

fn plain_init<'a>(program: &'a Program<'a>) -> &'a Expression<'a> {
    let Statement::VariableDeclaration(declaration) = &program.body[0] else {
        panic!("plain const");
    };
    declaration.declarations[0]
        .init
        .as_ref()
        .expect("plain initializer")
}

fn call_with_receiver<'a>(program: &'a Program<'a>, name: &str) -> &'a CallExpression<'a> {
    let Expression::CallExpression(call) = expression_with_receiver(program, name) else {
        panic!("call");
    };
    call.as_ref()
}

fn expression_with_receiver<'a>(program: &'a Program<'a>, name: &str) -> &'a Expression<'a> {
    program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::ExpressionStatement(statement) = statement else {
                return None;
            };
            let expression = crate::codebase::ts_source::unwrap_ts_wrappers(&statement.expression);
            let Expression::CallExpression(call) = expression else {
                return None;
            };
            (receiver(call) == name).then_some(expression)
        })
        .expect("call")
}

fn receiver(call: &CallExpression<'_>) -> String {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return String::new();
    };
    let Expression::Identifier(identifier) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
    else {
        return String::new();
    };
    identifier.name.to_string()
}
