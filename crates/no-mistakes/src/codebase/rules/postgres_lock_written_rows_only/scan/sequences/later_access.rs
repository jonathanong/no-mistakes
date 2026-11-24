use super::{transaction_scope, EmbeddedSqlCall, SqlStatementFileFacts, Transaction};
use crate::codebase::postgres::{decoded_parts, idents::relation_suffix_key};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct LaterAccess<'a> {
    reads: Accesses<'a>,
    writes: Accesses<'a>,
}

impl<'a> LaterAccess<'a> {
    pub(super) fn collect(
        calls: &'a [EmbeddedSqlCall],
        grouped: &BTreeMap<usize, Vec<&'a SqlStatementFileFacts>>,
    ) -> Self {
        let mut accesses = Self::default();
        for (&index, versions) in grouped {
            let call = &calls[index];
            let Some(transaction) = transaction_scope(call) else {
                continue;
            };
            if call.helper_definition {
                continue;
            }
            let position = call.execution_range.0;
            for facts in versions.iter().filter(|facts| !facts.parse_failed) {
                let reads = facts
                    .selects
                    .iter()
                    .flat_map(|select| select.tables.iter().map(String::as_str))
                    .chain(
                        facts
                            .updates
                            .iter()
                            .chain(&facts.deletes)
                            .flatten()
                            .map(|relation| relation.table.as_str()),
                    );
                for relation in reads {
                    accesses.reads.insert(transaction, position, relation);
                    for (dot, _) in relation.match_indices('.') {
                        accesses
                            .reads
                            .insert_suffix(transaction, position, &relation[dot + 1..]);
                    }
                }
                let writes = facts
                    .writes
                    .iter()
                    .map(|write| write.table.as_str())
                    .chain(
                        facts
                            .lock_writes
                            .writes
                            .iter()
                            .filter_map(|write| write.relation.as_deref()),
                    )
                    .chain(
                        facts
                            .lock_writes
                            .table_locks
                            .iter()
                            .filter_map(Option::as_deref),
                    );
                for relation in writes {
                    accesses.writes.insert(transaction, position, relation);
                    let leaf = relation_suffix_key(relation);
                    if leaf != relation {
                        accesses.writes.insert_suffix(transaction, position, leaf);
                    }
                }
            }
        }
        accesses
    }

    pub(super) fn may_follow(&self, tx: Transaction, end: u32, relation: &str) -> bool {
        if self
            .writes
            .may_follow(tx, end, relation, relation_suffix_key(relation))
        {
            return true;
        }
        let parts = decoded_parts(relation);
        let Some(leaf) = parts.last() else {
            return false;
        };
        self.reads.may_follow(tx, end, &parts.join("."), leaf)
    }
}

#[derive(Default)]
struct Accesses<'a> {
    exact: Positions<'a>,
    qualified: Positions<'a>,
}

type Positions<'a> = BTreeMap<(u32, &'a str), Latest>;

impl<'a> Accesses<'a> {
    fn insert(&mut self, tx: Transaction, position: u32, relation: &'a str) {
        Latest::record(&mut self.exact, tx, position, relation);
    }

    fn insert_suffix(&mut self, tx: Transaction, position: u32, leaf: &'a str) {
        Latest::record(&mut self.qualified, tx, position, leaf);
    }

    fn may_follow(&self, tx: Transaction, end: u32, relation: &str, leaf: &str) -> bool {
        let possible = if relation == leaf {
            self.qualified.get(&(tx.0, leaf))
        } else {
            self.exact.get(&(tx.0, leaf))
        };
        self.exact
            .get(&(tx.0, relation))
            .into_iter()
            .chain(possible)
            .any(|latest| latest.scope != Some(tx.1) || latest.position > end)
    }
}

struct Latest {
    position: u32,
    scope: Option<u32>,
}

impl Latest {
    fn record<'a>(
        positions: &mut Positions<'a>,
        tx: Transaction,
        position: u32,
        relation: &'a str,
    ) {
        let latest = positions.entry((tx.0, relation)).or_insert(Self {
            position,
            scope: Some(tx.1),
        });
        latest.position = latest.position.max(position);
        if latest.scope != Some(tx.1) {
            latest.scope = None;
        }
    }
}
