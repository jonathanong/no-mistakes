use super::*;
use oxc_ast_visit::Visit;

pub(super) struct PhaseWrite {
    pub(super) target: u32,
    pub(super) name: String,
    pub(super) path: ValueMutationPath,
    pub(super) phase: u32,
}
pub(super) struct PhaseWalk<'c> {
    pub(super) collector: Option<&'c ImportCollector>,
    pub(super) phase: u32,
    pub(super) cursor: u32,
    pub(super) segments: Vec<(u32, u32, u32)>,
    pub(super) writes: Vec<PhaseWrite>,
    pub(super) nested_pattern: bool,
    pub(super) pattern_depth: usize,
    pub(super) collect_reference_effects: bool,
    pub(super) effects: Option<std::collections::BTreeSet<(u32, u32)>>,
    pub(super) fresh_phases: Option<std::collections::BTreeSet<u32>>,
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
    fn skip(&mut self, span: oxc_span::Span) {
        self.segment(span.start);
        self.cursor = span.end;
    }
}
impl<'a> Visit<'a> for PhaseWalk<'_> {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        let class =
            self.collect_reference_effects && matches!(expression, Expression::ClassExpression(_));
        if class {
            self.effects
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
            self.effects
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
            self.expression(&assignment.right);
            self.write(&assignment.left);
            if self.collect_reference_effects && receiver_replacement_is_fresh(assignment) {
                self.fresh_phases
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
        self.write(target);
        self.cursor = update.span.end;
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
