use super::*;

pub(super) fn direct_arrow(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            direct_arrow(&parenthesized.expression)
        }
        _ => false,
    }
}

impl ImportCollector {
    pub(super) fn parameter_type_reference(
        &self,
        param: &oxc_ast::ast::FormalParameter<'_>,
    ) -> Option<String> {
        let oxc_ast::ast::TSType::TSTypeReference(reference) =
            &param.type_annotation.as_ref()?.type_annotation
        else {
            return None;
        };
        let name = type_reference_name(reference)?;
        let root = name.split('.').next()?;
        (!self.type_parameter_shadows(root) && !self.type_binding_shadows(root)).then_some(name)
    }
}
