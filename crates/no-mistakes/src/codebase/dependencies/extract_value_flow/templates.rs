use super::*;

pub(super) fn tag_offset(tagged: &TaggedTemplateExpression<'_>) -> u32 {
    if simple_callee_name(&tagged.tag).is_some() {
        tagged.span.start
    } else {
        tagged.quasi.span.start
    }
}
impl ImportCollector {
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
        let mut arguments = vec![ValueExpression::Unknown];
        arguments.extend(
            tagged
                .quasi
                .expressions
                .iter()
                .map(|value| self.value_expression(value)),
        );
        self.value_flow.calls.push(ValueCall {
            offset: tag_offset(tagged),
            end: tagged.span.end,
            source_callee: simple_callee_name(&tagged.tag)
                .unwrap_or_else(|| "<unknown>".to_string()),
            callee: self.value_expression(&tagged.tag),
            arguments: std::sync::Arc::new(arguments),
            bound: false,
            spread_at: None,
        });
    }
}
