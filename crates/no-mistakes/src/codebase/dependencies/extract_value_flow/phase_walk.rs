use super::*;
use oxc_ast_visit::Visit;

pub(super) struct PhaseWrite {
    pub(super) target: u32,
    pub(super) name: String,
    pub(super) path: ValueMutationPath,
    pub(super) phase: u32,
}
#[derive(Default)]
pub(super) struct ReferencePhaseEvents {
    pub(super) opaque: Option<std::collections::BTreeSet<(u32, u32)>>,
    pub(super) fresh: Option<std::collections::BTreeSet<u32>>,
}
pub(super) struct RootPhaseWrite {
    pub(super) scope: usize,
    pub(super) name: String,
    pub(super) phase: u32,
    pub(super) value: ValueExpression,
    pub(super) guaranteed: bool,
}
pub(super) struct PhaseWalk<'c> {
    pub(super) collector: Option<&'c ImportCollector>,
    pub(super) phase: u32,
    pub(super) cursor: u32,
    pub(super) segments: Vec<(u32, u32, u32)>,
    pub(super) writes: Vec<PhaseWrite>,
    pub(super) root_writes: Vec<RootPhaseWrite>,
    pub(super) collect_root_origins: bool,
    #[cfg(feature = "test-instrumentation")]
    pub(super) root_visits: usize,
    pub(super) conditional: bool,
    pub(super) nested_pattern: bool,
    pub(super) pattern_depth: usize,
    pub(super) collect_reference_effects: bool,
    pub(super) proof_events: ReferencePhaseEvents,
}
impl PhaseWalk<'_> {
    pub(super) fn segment(&mut self, end: u32) {
        if self.cursor < end {
            self.segments.push((self.cursor, end, self.phase));
        }
        self.cursor = end;
    }
    pub(super) fn expression<'a>(&mut self, expression: &Expression<'a>) {
        let cursor = self.cursor;
        self.cursor = expression.span().start;
        self.visit_expression(expression);
        self.segment(expression.span().end);
        self.cursor = cursor;
    }
    pub(super) fn conditional_expression(&mut self, expression: &Expression<'_>) {
        let previous = std::mem::replace(&mut self.conditional, true);
        self.expression(expression);
        self.conditional = previous;
    }
    fn skip(&mut self, span: oxc_span::Span) {
        self.segment(span.start);
        self.cursor = span.end;
    }
}
impl<'a> Visit<'a> for PhaseWalk<'_> {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        #[cfg(feature = "test-instrumentation")]
        if self.collect_root_origins {
            self.root_visits += 1;
        }
        let class =
            self.collect_reference_effects && matches!(expression, Expression::ClassExpression(_));
        if class {
            self.proof_events
                .opaque
                .get_or_insert_with(Default::default)
                .insert((self.phase, expression.span().start));
        }
        oxc_ast_visit::walk::walk_expression(self, expression);
        if self.collect_reference_effects
            && !self
                .collector
                .is_some_and(|collector| collector.known_own_data_read(expression))
            && matches!(
                expression,
                Expression::CallExpression(_)
                    | Expression::NewExpression(_)
                    | Expression::TaggedTemplateExpression(_)
                    | Expression::StaticMemberExpression(_)
                    | Expression::ComputedMemberExpression(_)
                    | Expression::AwaitExpression(_)
                    | Expression::YieldExpression(_)
                    | Expression::ImportExpression(_)
                    | Expression::BinaryExpression(_)
                    | Expression::UnaryExpression(_)
                    | Expression::UpdateExpression(_)
                    | Expression::TemplateLiteral(_)
                    | Expression::ClassExpression(_)
                    | Expression::JSXElement(_)
                    | Expression::JSXFragment(_)
            )
        {
            self.proof_events
                .opaque
                .get_or_insert_with(Default::default)
                .insert((self.phase, expression.span().end));
        }
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        self.segment(assignment.span.start);
        // Only assignment evaluation is reordered here. The enclosing ordinary
        // expression retains its source-order read positions.
        if matches!(
            assignment.left,
            AssignmentTarget::ArrayAssignmentTarget(_)
                | AssignmentTarget::ObjectAssignmentTarget(_)
        ) {
            self.expression(&assignment.right);
            self.target(&assignment.left);
        } else {
            self.reference(&assignment.left);
            if assignment.operator.is_logical() {
                self.conditional_expression(&assignment.right);
            } else {
                self.expression(&assignment.right);
            }
            let value = (assignment.operator.is_assign() || assignment.operator.is_logical())
                .then_some(&assignment.right);
            self.write(&assignment.left, value, !assignment.operator.is_logical());
            if self.collect_reference_effects && receiver_replacement_is_fresh(assignment) {
                self.proof_events
                    .fresh
                    .get_or_insert_with(Default::default)
                    .insert(self.phase);
            }
        }
        self.cursor = assignment.span.end;
    }
    fn visit_update_expression(&mut self, update: &oxc_ast::ast::UpdateExpression<'a>) {
        self.segment(update.span.start);
        let target = update.argument.as_assignment_target();
        self.reference(target);
        self.write(target, None, true);
        self.cursor = update.span.end;
    }
    fn visit_conditional_expression(
        &mut self,
        expression: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        self.visit_expression(&expression.test);
        let previous = std::mem::replace(&mut self.conditional, true);
        self.visit_expression(&expression.consequent);
        self.visit_expression(&expression.alternate);
        self.conditional = previous;
    }
    fn visit_logical_expression(&mut self, expression: &oxc_ast::ast::LogicalExpression<'a>) {
        self.visit_expression(&expression.left);
        let previous = std::mem::replace(&mut self.conditional, true);
        self.visit_expression(&expression.right);
        self.conditional = previous;
    }
    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        // The normal collector owns class-local bindings and initializer
        // execution contexts. This borrowed outer scope cannot resolve class
        // assignments; do not turn deferred or shadowed writes into root facts.
        let collector = self.collector.take();
        walk::walk_class(self, class);
        self.collector = collector;
    }
    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.skip(function.span);
    }
    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.skip(arrow.span);
    }
}
