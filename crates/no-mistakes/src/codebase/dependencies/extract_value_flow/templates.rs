use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TaggedTemplateInfo {
    pub(crate) literal_arguments: Vec<usize>,
    pub(crate) opaque_parameter_defaults: Vec<usize>,
}

pub(super) fn tag_offset(tagged: &TaggedTemplateExpression<'_>) -> u32 {
    if simple_callee_name(&tagged.tag).is_some() {
        tagged.span.start
    } else {
        tagged.quasi.span.start
    }
}
impl ImportCollector {
    pub(super) fn note_text_parameter_default(
        &mut self,
        first: usize,
        parameter: &oxc_ast::ast::FormalParameter<'_>,
        value: &ValueExpression,
    ) {
        let initializer = match &parameter.pattern {
            BindingPattern::AssignmentPattern(assignment) => Some(&assignment.right),
            _ => parameter.initializer.as_ref(),
        };
        if matches!(value, ValueExpression::Unknown)
            && initializer.is_some_and(|value| !literal_argument(value))
        {
            self.opaque_text_parameter_defaults
                .extend(first..self.value_flow.parameters.len());
        }
    }

    fn visible_opaque_parameter_defaults(&self) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .opaque_text_parameter_defaults
            .iter()
            .filter_map(|index| {
                let parameter = self.value_flow.parameters.get(*index)?;
                self.lexical_scope_ids
                    .contains(&parameter.scope)
                    .then_some(*index)
            })
            .collect();
        indices.sort_unstable();
        indices
    }

    pub(in super::super) fn visit_value_tagged<'a>(
        &mut self,
        tagged: &TaggedTemplateExpression<'a>,
    ) {
        self.record_value_tagged(tagged);
        let before = self.function_calls.len();
        visit_tagged_template_expression_with_imports(self, tagged);
        self.record_per_item_invocations(before);
        walk::walk_tagged_template_expression(self, tagged);
    }

    pub(in super::super) fn record_value_tagged(&mut self, tagged: &TaggedTemplateExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        self.value_flow.tagged_templates.insert(
            (tag_offset(tagged), tagged.span.end),
            TaggedTemplateInfo {
                opaque_parameter_defaults: self.visible_opaque_parameter_defaults(),
                literal_arguments: tagged
                    .quasi
                    .expressions
                    .iter()
                    .enumerate()
                    .filter_map(|(index, value)| literal_argument(value).then_some(index + 1))
                    .collect(),
            },
        );
        let mut arguments = vec![ValueExpression::Unknown];
        arguments.extend(
            tagged
                .quasi
                .expressions
                .iter()
                .map(|value| self.value_expression(value)),
        );
        self.value_flow.calls.push(ValueCall {
            context: self.current_function_id(),
            offset: tag_offset(tagged),
            end: tagged.span.end,
            source_callee: simple_callee_name(&tagged.tag)
                .unwrap_or_else(|| "<unknown>".to_string()),
            callee: self.value_expression(&tagged.tag),
            arguments: std::sync::Arc::new(arguments),
            last_argument_literal: false,
            bound: false,
            spread_at: None,
            once: false,
            repeated: self.repeat_depth > self.call_repeat_base,
        });
    }
}

fn literal_argument(value: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(value) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}
