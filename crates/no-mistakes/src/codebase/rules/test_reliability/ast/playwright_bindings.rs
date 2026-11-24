use super::bindings::Bindings;
use crate::codebase::dependencies::extract::ExportedBinding;
use crate::codebase::rules::test_reliability::playwright_provenance::{
    FixtureInitializer, FixtureInitializers,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, Program};
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
        if !scope.symbol_flags(symbol).is_const_variable() {
            continue;
        }
        let Some(initializer) = semantic
            .nodes()
            .kind(scope.symbol_declaration(symbol))
            .as_variable_declarator()
            .filter(|declaration| declaration.id.is_binding_identifier())
            .and_then(|declaration| declaration.init.as_ref())
        else {
            continue;
        };
        if let Some(import) = bindings.playwright_value_origin(initializer) {
            result.0.insert(export.local.clone(), import);
        }
    }
    result
}

impl Bindings<'_, '_> {
    pub(super) fn playwright_value_origin(
        &self,
        value: &Expression<'_>,
    ) -> Option<FixtureInitializer> {
        let mut derived_at = None;
        let mut value = unwrap_ts_wrappers(value);
        loop {
            if let Some(member) = value.as_member_expression() {
                let property = member.static_property_name()?.as_str()?;
                let Expression::Identifier(object) = unwrap_ts_wrappers(member.object()) else {
                    return None;
                };
                return Some(FixtureInitializer {
                    import: self.import_origin(self.symbol(object)?, Some(property))?,
                    derived_at,
                });
            }
            match value {
                Expression::Identifier(identifier) => {
                    if let Some(import) = self.import_origin(self.symbol(identifier)?, None) {
                        return Some(FixtureInitializer { import, derived_at });
                    }
                    value = unwrap_ts_wrappers(self.constant_initializer(identifier)?);
                }
                Expression::CallExpression(call) if !call.optional => {
                    let member = unwrap_ts_wrappers(&call.callee).as_member_expression()?;
                    if member.static_property_name()?.as_str()? != "extend" {
                        return None;
                    }
                    derived_at.get_or_insert(call.span.start);
                    value = unwrap_ts_wrappers(member.object());
                }
                _ => return None,
            }
        }
    }
}
