use super::super::conditions::literal;
use super::super::scope::ScopeSnapshot;
use super::{LookupHistory, ANY, FALSE, TRUE};
use sqlparser::ast::{Expr, Value};

impl LookupHistory {
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

    pub(in crate::statements::keyed_lookups) fn permit_alternative(&mut self, other: &Self) {
        self.active.columns.retain(|column, allowed| {
            let Some(alternative) = other.active.columns.get(column) else {
                return false;
            };
            *allowed |= alternative;
            true
        });
    }
}
