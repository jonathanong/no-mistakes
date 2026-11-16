use super::*;

impl ImportCollector {
    pub(crate) fn note_widened_string_assertion(&mut self, expression: &Expression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let (annotation, operand) = match expression {
            Expression::TSAsExpression(assertion) => {
                (&assertion.type_annotation, &assertion.expression)
            }
            Expression::TSTypeAssertion(assertion) => {
                (&assertion.type_annotation, &assertion.expression)
            }
            _ => return,
        };
        if matches!(
            self.annotation_shape(annotation),
            ValueType::FiniteStringLiterals
        ) {
            return;
        }
        let value = self.value_expression(operand);
        record_named_offsets(&value, &mut self.value_flow.widened_string_assertions);
        // SQL references retain the assertion start for angle-bracket casts.
        self.value_flow
            .widened_string_assertions
            .insert(expression.span().start);
    }
}

fn record_named_offsets(value: &ValueExpression, offsets: &mut FxHashSet<u32>) {
    match value {
        ValueExpression::Named { offset, .. } => {
            offsets.insert(*offset);
        }
        ValueExpression::Member(inner, _) | ValueExpression::Awaited(inner) => {
            record_named_offsets(inner, offsets);
        }
        ValueExpression::Derived(parts) => {
            for part in parts.iter() {
                record_named_offsets(part, offsets);
            }
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            record_named_offsets(left, offsets);
            record_named_offsets(right, offsets);
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            for value in fields.values() {
                record_named_offsets(value, offsets);
            }
        }
        _ => {}
    }
}
