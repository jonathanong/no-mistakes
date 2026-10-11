use super::ScopeVisitor;
use oxc_ast::ast::TemplateLiteral;
mod captures;
use captures::CapturedBuilders;

pub(super) fn mutates_captured_fragment(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
) -> bool {
    let Some((last, earlier)) = template.expressions.split_last() else {
        return false;
    };
    let mut captured = CapturedBuilders::default();
    for expression in earlier {
        if visitor.expression_mutates_builder(expression) {
            return true;
        }
        captured.extend(visitor, expression);
    }
    captured.mutated_by(visitor, last)
}
