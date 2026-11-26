use super::*;

impl<'s, 'a> Collector<'s, 'a> {
    pub(super) fn url(&self, created: &NewExpression<'a>) -> Option<Value> {
        if !self.global(&created.callee, "URL") {
            return None;
        }
        let [path, base] = created.arguments.as_slice() else {
            return None;
        };
        let Expression::StringLiteral(path) = path.as_expression()?.get_inner_expression() else {
            return None;
        };
        let Expression::StaticMemberExpression(base) = base.as_expression()?.get_inner_expression()
        else {
            return None;
        };
        if base.optional
            || base.property.name != "url"
            || !matches!(
                base.object.get_inner_expression(),
                Expression::ImportMeta(_)
            )
        {
            return None;
        }
        let specifier = crate::ast::js_literal(path).into_owned();
        if !(specifier.starts_with("./") || specifier.starts_with("../")) {
            return None;
        }
        let lower = specifier.to_ascii_lowercase();
        if ["%2f", "%5c", "%00"]
            .iter()
            .any(|escape| lower.contains(escape))
        {
            return None;
        }
        Some(Value {
            specifier,
            relative_url: true,
            pathname: false,
            object: true,
        })
    }

    pub(super) fn global(&self, expression: &Expression<'a>, name: &str) -> bool {
        let Expression::Identifier(identifier) = expression.get_inner_expression() else {
            return false;
        };
        identifier.name == name
            && identifier.reference_id.get().is_some_and(|id| {
                !super::super::modules::loads::has_runtime_binding(
                    self.semantic.scoping().get_reference(id),
                    self.semantic,
                )
            })
    }
}
