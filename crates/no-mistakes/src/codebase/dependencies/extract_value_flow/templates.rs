use super::*;

#[path = "text_origin_facts.rs"]
pub(super) mod origins;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TaggedTemplateInfo {
    pub(crate) literal_arguments: Vec<usize>,
    pub(crate) opaque_parameter_defaults: Vec<usize>,
    pub(crate) origins: Option<std::sync::Arc<origins::TextOrigins>>,
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
            _ => parameter.initializer.as_deref(),
        };
        if let Some(initializer) = initializer {
            self.note_opaque_text_default(first, initializer, value);
        }
    }

    pub(super) fn note_opaque_text_default(
        &mut self,
        first: usize,
        initializer: &Expression<'_>,
        value: &ValueExpression,
    ) {
        if matches!(value, ValueExpression::Unknown) && !literal_argument(initializer) {
            self.opaque_text_parameter_defaults
                .extend(first..self.value_flow.parameters.len());
        }
    }

    pub(super) fn finalize_text_parameter_defaults(&mut self) {
        let mut indices: Vec<_> = self
            .opaque_text_parameter_defaults
            .iter()
            .copied()
            .collect();
        indices.sort_unstable();
        if self.collect_identifier_provenance {
            self.value_flow.identifier_text_origins = self
                .text_origins
                .identifier_bindings()
                .map(std::sync::Arc::new);
        }
        let origins = std::sync::Arc::new(std::mem::take(&mut self.text_origins));
        for info in self.value_flow.tagged_templates.values_mut() {
            info.opaque_parameter_defaults.clone_from(&indices);
            info.origins = Some(std::sync::Arc::clone(&origins));
        }
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
                origins: None,
                opaque_parameter_defaults: Vec::new(),
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
