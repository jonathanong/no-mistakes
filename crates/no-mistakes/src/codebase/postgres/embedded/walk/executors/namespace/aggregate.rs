use super::super::ScopeVisitor;
use oxc_ast::ast::{Expression, ObjectPropertyKind};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn revoke_aggregate_values(
        &mut self,
        expression: &Expression<'_>,
    ) {
        match expression {
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    let value = match property {
                        ObjectPropertyKind::ObjectProperty(value) => &value.value,
                        ObjectPropertyKind::SpreadProperty(value) => &value.argument,
                    };
                    self.revoke_query_receiver(value);
                }
            }
            Expression::ArrayExpression(array) => {
                for value in &array.elements {
                    if let Some(value) = value.as_expression() {
                        self.revoke_query_receiver(value);
                    }
                }
            }
            _ => {}
        }
    }
}
