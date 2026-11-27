use super::*;
use sqlparser::ast::{JoinConstraint, JoinOperator, TableWithJoins};
mod tables;
impl Collector<'_, '_> {
    pub(super) fn from(
        &mut self,
        table: &TableWithJoins,
        scope: usize,
        env: &CteEnvironment,
    ) -> Vec<usize> {
        let mut left = self.relation(&table.relation, scope, env);
        for join in &table.joins {
            let right = self.relation(&join.relation, scope, env);
            use PostgresSqlQueryJoinKind as K;
            let (kind, constraint) = match &join.join_operator {
                JoinOperator::Join(c) | JoinOperator::Inner(c) => (K::Inner, Some(c)),
                JoinOperator::Left(c) | JoinOperator::LeftOuter(c) => (K::Left, Some(c)),
                JoinOperator::Right(c) | JoinOperator::RightOuter(c) => (K::Right, Some(c)),
                JoinOperator::FullOuter(c) => (K::Full, Some(c)),
                JoinOperator::CrossJoin(c) => (K::Cross, Some(c)),
                JoinOperator::Semi(c) | JoinOperator::LeftSemi(c) | JoinOperator::RightSemi(c) => {
                    (K::Semi, Some(c))
                }
                JoinOperator::Anti(c) | JoinOperator::LeftAnti(c) | JoinOperator::RightAnti(c) => {
                    (K::Anti, Some(c))
                }
                _ => {
                    self.unsupported(
                        scope,
                        PostgresSqlQueryClause::From,
                        "join operator",
                        join.span(),
                    );
                    (K::Other, None)
                }
            };
            let id = self.facts.joins.len();
            let (label, using_columns) = match constraint {
                Some(JoinConstraint::On(_)) => ("on", Vec::new()),
                Some(JoinConstraint::Using(cols)) => ("using", cols.iter().map(name).collect()),
                Some(JoinConstraint::Natural) => ("natural", Vec::new()),
                _ => ("none", Vec::new()),
            };
            let mandatory = matches!(kind, K::Inner | K::Semi);
            self.facts.joins.push(PostgresSqlQueryJoin {
                id,
                scope_id: scope,
                kind,
                left: left.clone(),
                right: right.clone(),
                constraint: label.into(),
                using_columns,
                span: self.locations.span(join.span()),
            });
            if let Some(JoinConstraint::On(expr)) = constraint {
                // JOIN ON sees its operands and outer scopes, not preceding comma items.
                let names = std::mem::take(&mut self.states[scope].names);
                self.states[scope].names = names
                    .iter()
                    .filter_map(|(key, ids)| {
                        let visible: Vec<_> = ids
                            .iter()
                            .copied()
                            .filter(|id| left.contains(id) || right.contains(id))
                            .collect();
                        (!visible.is_empty()).then(|| (key.clone(), visible))
                    })
                    .collect();
                self.expr(
                    expr,
                    scope,
                    PostgresSqlQueryClause::JoinOn,
                    Some(id),
                    Self::predicate_context(mandatory),
                    env,
                );
                self.states[scope].names = names;
            }
            left.extend(right);
        }
        left
    }
}
