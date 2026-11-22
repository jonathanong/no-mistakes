use super::*;
use crate::codebase::postgres::idents::relation_suffix_key;

#[derive(Default)]
pub(super) struct PotentialWrites<'a> {
    by_transaction: BTreeMap<Transaction, LatestWrites<'a>>,
    by_scope: BTreeMap<u32, LatestWrites<'a>>,
    unknown_transactions: BTreeMap<u32, LatestWrites<'a>>,
}

impl<'a> PotentialWrites<'a> {
    pub fn collect(
        candidates: impl Iterator<Item = (&'a EmbeddedSqlCall, &'a [&'a SqlStatementFileFacts])>,
    ) -> Self {
        let mut potential = Self::default();
        for (call, versions) in candidates.filter(|(call, _)| !known_autocommit(call)) {
            let positions = if call.execution_awaited && call.lock_helper.is_none() {
                call.execution_range.1
            } else {
                u32::MAX
            };
            for facts in versions.iter().filter(|facts| !facts.parse_failed) {
                for write in &facts.lock_writes.writes {
                    let relation = write.relation.as_deref();
                    potential
                        .by_scope
                        .entry(call.execution_scope)
                        .or_default()
                        .insert(relation, positions);
                    if let Some(transaction) = transaction_scope(call) {
                        potential
                            .by_transaction
                            .entry(transaction)
                            .or_default()
                            .insert(relation, positions);
                    } else {
                        potential
                            .unknown_transactions
                            .entry(call.execution_scope)
                            .or_default()
                            .insert(relation, positions);
                    }
                }
            }
        }
        potential
    }

    pub fn may_follow(&self, call: &EmbeddedSqlCall, relation: Option<&str>) -> bool {
        let after = call.execution_range.1;
        if let Some(transaction) = transaction_scope(call) {
            self.by_transaction
                .get(&transaction)
                .is_some_and(|writes| writes.may_follow(relation, after))
                || self
                    .unknown_transactions
                    .get(&call.execution_scope)
                    .is_some_and(|writes| writes.may_follow(relation, after))
        } else {
            self.by_scope
                .get(&call.execution_scope)
                .is_some_and(|writes| writes.may_follow(relation, after))
        }
    }

    pub fn may_follow_lock(&self, call: &EmbeddedSqlCall, facts: &SqlStatementFileFacts) -> bool {
        let mut inside_batch = LatestWrites::default();
        for statement in facts.lock_writes.share_statements.iter().rev() {
            if statement.locks.iter().any(|relation| {
                self.may_follow(call, relation.as_deref())
                    || inside_batch.may_follow(relation.as_deref(), 0)
            }) {
                return true;
            }
            for write in &facts.lock_writes.writes[statement.writes.clone()] {
                inside_batch.insert(write.relation.as_deref(), 1);
            }
        }
        false
    }
}

#[derive(Default)]
struct LatestWrites<'a> {
    any: u32,
    unresolved: u32,
    exact: BTreeMap<&'a str, u32>,
    qualified_leaves: BTreeMap<&'a str, u32>,
}

impl<'a> LatestWrites<'a> {
    fn insert(&mut self, relation: Option<&'a str>, position: u32) {
        self.any = self.any.max(position);
        let Some(relation) = relation else {
            self.unresolved = self.unresolved.max(position);
            return;
        };
        let exact = self.exact.entry(relation).or_default();
        *exact = (*exact).max(position);
        let leaf = relation_suffix_key(relation);
        if leaf != relation {
            let qualified = self.qualified_leaves.entry(leaf).or_default();
            *qualified = (*qualified).max(position);
        }
    }

    fn may_follow(&self, relation: Option<&str>, after: u32) -> bool {
        let Some(relation) = relation else {
            return self.any > after;
        };
        let leaf = relation_suffix_key(relation);
        let suffix_position = if leaf == relation {
            self.qualified_leaves.get(leaf)
        } else {
            self.exact.get(leaf)
        };
        self.unresolved > after
            || self
                .exact
                .get(relation)
                .is_some_and(|position| *position > after)
            || suffix_position.is_some_and(|position| *position > after)
    }
}
