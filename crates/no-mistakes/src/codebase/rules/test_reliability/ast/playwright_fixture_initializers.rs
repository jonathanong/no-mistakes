use super::bindings::Bindings;
use super::playwright_bindings::ValueTrace;
use crate::codebase::dependencies::extract::ExportedBinding;
use crate::codebase::rules::test_reliability::playwright_provenance::FixtureInitializers;
use crate::fx::FxHashSet;
use oxc_ast::ast::Program;
use oxc_semantic::Semantic;

pub(crate) fn fixture_initializers<'a>(
    program: &'a Program<'a>,
    semantic: &Semantic<'a>,
    exports: &[ExportedBinding],
) -> FixtureInitializers {
    let mut result = FixtureInitializers::default();
    if exports.is_empty() {
        return result;
    }
    let bindings = Bindings::new(program, semantic, false, true);
    let scope = semantic.scoping();
    for export in exports.iter().filter(|export| export.specifier.is_none()) {
        let Some(symbol) = scope.get_binding(scope.root_scope_id(), export.local.as_str().into())
        else {
            continue;
        };
        let mut visited = FxHashSet::default();
        if let Some(summary) = bindings.wrapper_summary(symbol, 0, &mut visited) {
            result.wrappers.insert(export.local.clone(), summary);
            continue;
        }
        let trace = if bindings.is_function_def(symbol) {
            bindings.factory_return_origin(symbol, 0)
        } else if let Some(declaration) = semantic
            .nodes()
            .kind(scope.symbol_declaration(symbol))
            .as_variable_declarator()
        {
            if scope.symbol_flags(symbol).is_const_variable()
                && declaration.id.is_binding_identifier()
            {
                declaration.init.as_ref().map_or(ValueTrace::None, |init| {
                    bindings.playwright_value_origin(init)
                })
            } else {
                bindings.non_const_or_destructured_trace(symbol)
            }
        } else {
            ValueTrace::None
        };
        match trace {
            ValueTrace::Resolved(origin) => {
                result.values.insert(export.local.clone(), origin);
            }
            ValueTrace::Unresolved(reason, candidates) => {
                result
                    .uncertain
                    .insert(export.local.clone(), (reason, candidates));
            }
            ValueTrace::None => {}
        }
    }
    result
}
