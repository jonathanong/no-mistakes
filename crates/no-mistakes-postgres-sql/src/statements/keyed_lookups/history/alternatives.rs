use super::super::conditions::literal;
use super::super::scope::ScopeSnapshot;
use super::{boolean_test, guard_column, Column, LookupHistory, ANY, FALSE, TRUE};
use crate::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr, Value};

impl LookupHistory {
    pub(in crate::statements::keyed_lookups) fn restrict(
        &mut self,
        condition: &Expr,
        outer: &ScopeSnapshot,
        taken: bool,
    ) {
        if taken {
            if let Expr::BinaryOp {
                left,
                op: BinaryOperator::And,
                right,
            } = unwrap_expr(condition)
            {
                self.restrict(left, outer, true);
                self.restrict(right, outer, true);
                return;
            }
        }
        let (operand, test, _) = boolean_test(condition);
        let allowed = if taken { test } else { ANY ^ test };
        self.restrict_operand(operand, outer, allowed);
    }

    pub(in crate::statements::keyed_lookups) fn restrict_case(
        &mut self,
        operand: Option<&Expr>,
        test: &Expr,
        outer: &ScopeSnapshot,
        taken: bool,
    ) {
        if let Some(operand) = operand {
            if let Some(Value::Boolean(test)) = literal(test) {
                let matches = if *test { TRUE } else { FALSE };
                self.restrict_operand(operand, outer, if taken { matches } else { ANY ^ matches });
            }
        } else {
            self.restrict(test, outer, taken);
        }
    }

    pub(in crate::statements::keyed_lookups) fn with_guards(
        &mut self,
        guards: &Self,
        inspect: impl FnOnce(&mut Self),
    ) {
        let previous = std::mem::replace(&mut self.active, guards.active.clone());
        inspect(self);
        self.active = previous;
    }

    pub(in crate::statements::keyed_lookups) fn preserve_grouped_column(
        &mut self,
        position: usize,
        expr: &Expr,
        source: &ScopeSnapshot,
        grouped: &ScopeSnapshot,
    ) {
        if let Some(allowed) = guard_column(expr, source)
            .and_then(|column| self.active.columns.get(&column))
            .copied()
        {
            self.active
                .columns
                .insert(Column::Grouped(position), allowed);
            if let Some(column) = guard_column(expr, grouped) {
                self.active.columns.insert(column, allowed);
            }
        }
    }
}
