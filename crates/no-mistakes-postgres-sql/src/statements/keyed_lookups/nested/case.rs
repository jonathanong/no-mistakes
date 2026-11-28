use super::super::conditions::case_condition;
use super::super::LookupHistory;
use super::{walk_nested_with_context, Nested};
use sqlparser::ast::{CaseWhen, Expr};

impl Nested<'_> {
    pub(super) fn walk_case(
        &mut self,
        operand: Option<&Expr>,
        conditions: &[CaseWhen],
        fallback: Option<&Expr>,
    ) {
        if let Some(operand) = operand {
            self.walk_child(operand);
        }
        let outer = self.scopes.snapshot();
        let mut remaining = self.guards.guard_only();
        for branch in conditions {
            let test = case_condition(operand, &branch.condition);
            if test == Some(false) {
                continue;
            }
            walk_nested_with_context(
                &branch.condition,
                self.scopes,
                self.out,
                remaining.guard_only(),
                self.empty_input,
                self.windows,
            );
            let mut chosen = remaining.guard_only();
            if operand.is_none() && test.is_none() {
                chosen.restrict(&branch.condition, &outer, true);
                remaining.restrict(&branch.condition, &outer, false);
            }
            self.walk_case_arm(&branch.result, chosen);
            if test == Some(true) {
                return;
            }
        }
        if let Some(fallback) = fallback {
            self.walk_case_arm(fallback, remaining);
        }
    }

    fn walk_case_arm(&mut self, expr: &Expr, guards: LookupHistory) {
        if guards.is_reachable() {
            walk_nested_with_context(
                expr,
                self.scopes,
                self.out,
                guards,
                self.empty_input,
                self.windows,
            );
        } else {
            self.walk_eager_children(expr);
        }
    }
}
