use super::bindings::Bindings;
use crate::codebase::dependencies::extract::ExportedBinding;
use crate::codebase::rules::test_reliability::playwright_provenance::{
    FixtureInitializer, FixtureInitializers, ImportRef, WrapperSummary,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{Expression, Program};
use oxc_semantic::{Semantic, SymbolId};

const MAX_VALUE_DEPTH: usize = 8;

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
        if scope.symbol_flags(symbol).is_const_variable() {
            if let Some(initializer) = semantic
                .nodes()
                .kind(scope.symbol_declaration(symbol))
                .as_variable_declarator()
                .filter(|declaration| declaration.id.is_binding_identifier())
                .and_then(|declaration| declaration.init.as_ref())
            {
                if let Some(import) = bindings.playwright_value_origin(initializer) {
                    result.values.insert(export.local.clone(), import);
                    continue;
                }
            }
        }
        let mut visited = FxHashSet::default();
        if let Some(summary) = bindings.wrapper_summary(symbol, 0, &mut visited) {
            result.wrappers.insert(export.local.clone(), summary);
        } else if let Some(origin) = bindings.factory_return_origin(symbol, 0) {
            result.values.insert(export.local.clone(), origin);
        }
    }
    result
}

impl Bindings<'_, '_> {
    pub(super) fn playwright_value_origin(
        &self,
        value: &Expression<'_>,
    ) -> Option<FixtureInitializer> {
        self.playwright_value_origin_depth(value, 0)
    }

    pub(super) fn playwright_value_origin_depth(
        &self,
        value: &Expression<'_>,
        depth: usize,
    ) -> Option<FixtureInitializer> {
        if depth >= MAX_VALUE_DEPTH {
            return None;
        }
        let mut derived_at = None;
        let mut wrappers = Vec::new();
        let mut value = unwrap_ts_wrappers(value);
        loop {
            if let Some(member) = value.as_member_expression() {
                let property = member.static_property_name()?.as_str()?;
                let Expression::Identifier(object) = unwrap_ts_wrappers(member.object()) else {
                    return None;
                };
                return Some(FixtureInitializer {
                    import: self.import_origin(self.symbol(object)?, Some(property))?,
                    wrappers,
                    derived_at,
                });
            }
            match value {
                Expression::Identifier(identifier) => {
                    if let Some(import) = self.import_origin(self.symbol(identifier)?, None) {
                        return Some(FixtureInitializer {
                            import,
                            wrappers,
                            derived_at,
                        });
                    }
                    value = unwrap_ts_wrappers(self.constant_initializer(identifier)?);
                }
                Expression::CallExpression(call) if !call.optional => {
                    let callee = unwrap_ts_wrappers(&call.callee);
                    if let Some(member) = callee.as_member_expression() {
                        if member.static_property_name().and_then(|n| n.as_str()) == Some("extend")
                        {
                            derived_at.get_or_insert(call.span.start);
                            value = unwrap_ts_wrappers(member.object());
                            continue;
                        }
                    }
                    let mut visited = FxHashSet::default();
                    if let Some(summary) = self.local_callee_summary(callee, depth, &mut visited) {
                        super::playwright_wrappers::extend_unique(&mut wrappers, summary.wrappers);
                        derived_at = derived_at.or(summary.derived_at);
                        value = unwrap_ts_wrappers(
                            call.arguments.get(summary.param_index)?.as_expression()?,
                        );
                        continue;
                    }
                    return self.call_value_origin(call, callee, depth, wrappers, derived_at);
                }
                _ => return None,
            }
        }
    }

    pub(super) fn local_callee_summary(
        &self,
        callee: &Expression<'_>,
        depth: usize,
        visited: &mut FxHashSet<SymbolId>,
    ) -> Option<WrapperSummary> {
        let Expression::Identifier(identifier) = callee else {
            return None;
        };
        let symbol = self.symbol(identifier)?;
        if self.import_origin(symbol, None).is_some() {
            return None;
        }
        self.wrapper_summary(symbol, depth, visited)
    }

    pub(super) fn callee_import_origin(&self, callee: &Expression<'_>) -> Option<ImportRef> {
        match callee {
            Expression::Identifier(id) => self.import_origin(self.symbol(id)?, None),
            expr => {
                let member = expr.as_member_expression()?;
                let prop = member.static_property_name()?.as_str()?;
                let Expression::Identifier(obj) = unwrap_ts_wrappers(member.object()) else {
                    return None;
                };
                self.import_origin(self.symbol(obj)?, Some(prop))
            }
        }
    }
}
