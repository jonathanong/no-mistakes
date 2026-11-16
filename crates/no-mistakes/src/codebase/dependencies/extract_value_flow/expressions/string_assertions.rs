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
        let shape = self.annotation_shape(annotation);
        if matches!(&shape, ValueType::FiniteStringLiterals)
            || matches!(&shape, ValueType::Reference(name) if name == "const")
        {
            return;
        }
        let target = match shape {
            ValueType::Reference(name) => ValueType::Reference(name),
            _ => ValueType::Unknown,
        };
        let value = self.value_expression(operand);
        record_named_offsets(
            &value,
            &target,
            &mut self.value_flow.widened_string_assertions,
        );
        // SQL references retain the assertion start for angle-bracket casts.
        record_assertion_target(
            expression.span().start,
            &target,
            &mut self.value_flow.widened_string_assertions,
        );
    }
}

fn record_named_offsets(
    value: &ValueExpression,
    target: &ValueType,
    offsets: &mut FxHashMap<u32, Vec<ValueType>>,
) {
    match value {
        ValueExpression::Named { offset, .. } => {
            record_assertion_target(*offset, target, offsets);
        }
        ValueExpression::Member(inner, _) | ValueExpression::Awaited(inner) => {
            record_named_offsets(inner, target, offsets);
        }
        ValueExpression::Derived(parts) => {
            for part in parts.iter() {
                record_named_offsets(part, target, offsets);
            }
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            record_named_offsets(left, target, offsets);
            record_named_offsets(right, target, offsets);
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => {
            for value in fields.values() {
                record_named_offsets(value, target, offsets);
            }
        }
        _ => {}
    }
}

fn record_assertion_target(
    offset: u32,
    target: &ValueType,
    offsets: &mut FxHashMap<u32, Vec<ValueType>>,
) {
    let targets = offsets.entry(offset).or_default();
    if !targets.contains(target) {
        targets.push(target.clone());
    }
}
