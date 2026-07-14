use super::CheckFactPlan;
use crate::codebase::dependencies::extract::extract_import_facts_from_program;
use crate::codebase::ts_source::facts::TsFileFacts;

pub(crate) fn ts_facts(
    plan: &CheckFactPlan,
    source: Option<std::sync::Arc<str>>,
    program: &oxc_ast::ast::Program<'_>,
    parse_error: String,
) -> TsFileFacts {
    if !(plan.imports || plan.graph.imports || plan.graph.function_calls) {
        return TsFileFacts {
            parse_error: Some(parse_error),
            source: source.as_deref().map(str::to_owned),
            ..Default::default()
        };
    }
    let import_facts = extract_import_facts_from_program(program);
    TsFileFacts {
        parse_error: Some(parse_error),
        source: source.as_deref().map(str::to_owned),
        imports: import_facts.imports,
        function_calls: import_facts.function_calls,
        exported_functions: import_facts.exported_functions,
        unknown_callers: import_facts.unknown_callers,
        has_unknown_top_level_call: import_facts.has_unknown_top_level_call,
        ..Default::default()
    }
}
