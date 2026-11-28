use super::super::conditions::case_condition;
use super::super::LookupHistory;
use super::Nested;
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
            self.walk_child_with_guards(&branch.condition, remaining.guard_only());
            let mut chosen = remaining.guard_only();
            if test.is_none() {
                chosen.restrict_case(operand, &branch.condition, &outer, true);
                remaining.restrict_case(operand, &branch.condition, &outer, false);
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
            self.walk_child_with_guards(expr, guards);
        } else {
            self.walk_eager_children(expr);
        }
    }
}
