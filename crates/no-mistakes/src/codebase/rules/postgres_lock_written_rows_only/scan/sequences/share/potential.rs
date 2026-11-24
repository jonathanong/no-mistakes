use super::{key_writes::KeyWrites, *};
use crate::codebase::postgres::idents::relation_suffix_key;

#[derive(Default)]
pub(super) struct PotentialWrites<'a> {
    writes: ByScope<'a>,
    key_writes: ByScope<'a>,
}

#[derive(Default)]
struct ByScope<'a> {
    by_transaction: BTreeMap<Transaction, LatestWrites<'a>>,
    by_scope: BTreeMap<u32, LatestWrites<'a>>,
    unknown_transactions: BTreeMap<u32, LatestWrites<'a>>,
}

impl<'a> PotentialWrites<'a> {
    pub fn collect(
        candidates: impl Iterator<Item = (&'a EmbeddedSqlCall, &'a [&'a SqlStatementFileFacts])>,
        key_writes: &KeyWrites<'_, '_>,
    ) -> Self {
        let mut potential = Self::default();
        for (call, versions) in candidates.filter(|(call, _)| !known_autocommit(call)) {
            let positions = if call.execution_awaited && call.lock_helper.is_none() {
                call.execution_range.1
            } else {
                u32::MAX
            };
            for facts in versions.iter().filter(|facts| !facts.parse_failed) {
                let access = &facts.lock_writes;
                for write in &access.writes {
                    potential
                        .writes
                        .insert(call, write.relation.as_deref(), positions);
                }
                for statement in &access.share_statements {
                    for index in statement.writes.clone() {
                        if key_writes.matches(call, access, statement, index) {
                            potential.key_writes.insert(
                                call,
                                access.writes[index].relation.as_deref(),
                                positions,
                            );
                        }
                    }
                }
            }
        }
        potential
    }

    pub fn may_follow(
        &self,
        call: &EmbeddedSqlCall,
        relation: Option<&str>,
        key_share: bool,
    ) -> bool {
        let index = if key_share {
            &self.key_writes
        } else {
            &self.writes
        };
        index.may_follow(call, relation)
    }

    pub fn may_follow_lock(
        &self,
        call: &EmbeddedSqlCall,
        facts: &SqlStatementFileFacts,
        key_writes: &KeyWrites<'_, '_>,
    ) -> bool {
        let mut inside_batch = LatestWrites::default();
        let mut batch_key_writes = LatestWrites::default();
        for statement in facts.lock_writes.share_statements.iter().rev() {
            if statement.locks.iter().any(|lock| {
                if lock.mode == SequenceMode::Update {
                    return false;
                }
                let relation = lock
                    .row
                    .and_then(|index| facts.lock_writes.locks[index].relation.as_deref());
                let key_share = lock.mode == SequenceMode::KeyShare;
                self.may_follow(call, relation, key_share)
                    || if key_share {
                        &batch_key_writes
                    } else {
                        &inside_batch
                    }
                    .may_follow(relation, 0)
            }) {
                return true;
            }
            for index in statement.writes.clone() {
                let relation = facts.lock_writes.writes[index].relation.as_deref();
                inside_batch.insert(relation, 1);
                if key_writes.matches(call, &facts.lock_writes, statement, index) {
                    batch_key_writes.insert(relation, 1);
                }
            }
        }
        false
    }
}

impl<'a> ByScope<'a> {
    fn insert(&mut self, call: &EmbeddedSqlCall, relation: Option<&'a str>, position: u32) {
        self.by_scope
            .entry(call.execution_scope)
            .or_default()
            .insert(relation, position);
        if let Some(transaction) = transaction_scope(call) {
            self.by_transaction
                .entry(transaction)
                .or_default()
                .insert(relation, position);
        } else {
            self.unknown_transactions
                .entry(call.execution_scope)
                .or_default()
                .insert(relation, position);
        }
    }

    fn may_follow(&self, call: &EmbeddedSqlCall, relation: Option<&str>) -> bool {
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
