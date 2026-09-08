use super::CheckFactPlan;
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source;
use crate::codebase::ts_source::facts::TsFileFacts;

pub(crate) fn ts_facts(
    plan: &CheckFactPlan,
    source: Option<std::sync::Arc<str>>,
    parsed_source: &str,
    program: &oxc_ast::ast::Program<'_>,
    parse_error: String,
) -> TsFileFacts {
    if !(plan.imports || plan.graph.imports || plan.graph.function_calls) {
        return TsFileFacts {
            parse_error: Some(parse_error),
            source,
            ..Default::default()
        };
    }
    let import_facts = extract_import_facts_from_program_with_source(program, parsed_source);
    TsFileFacts {
        parse_error: Some(parse_error),
        source,
        imports: import_facts.imports,
        imported_bindings: import_facts.imported_bindings,
        exported_bindings: import_facts.exported_bindings,
        callable_aliases: import_facts.callable_aliases,
        star_reexport_specifiers: import_facts.star_reexport_specifiers,
        function_calls: import_facts.function_calls,
        unknown_calls: import_facts.unknown_calls,
        known_function_scopes: import_facts.known_function_scopes,
        callable_scopes: import_facts.callable_scopes,
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        ..Default::default()
    }
}
