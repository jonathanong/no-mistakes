use crate::playwright::ast;

pub(super) fn helper_argument_literals(
    call: &oxc_ast::ast::CallExpression<'_>,
    source: &str,
) -> Vec<String> {
    call.arguments
        .iter()
        .filter_map(|argument| match argument {
            oxc_ast::ast::Argument::StringLiteral(literal) => Some(literal.value.to_string()),
            oxc_ast::ast::Argument::TemplateLiteral(template) => {
                Some(ast::template_literal_text(template.as_ref(), source))
            }
            _ => None,
        })
        .collect()
}
