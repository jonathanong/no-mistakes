use super::*;

#[derive(Default)]
pub(super) struct LaterWrites<'a> {
    pub(super) relations: relations::Writes<'a>,
    all_rows: BTreeSet<&'a str>,
    unknown_rows: BTreeSet<&'a str>,
    columns: BTreeMap<&'a str, BTreeSet<&'a str>>,
    rows: BTreeMap<(&'a str, &'a str), BTreeSet<String>>,
}

impl<'a> LaterWrites<'a> {
    pub(super) fn insert(&mut self, row: &'a RowAccess, call: &'a EmbeddedSqlCall) {
        let Some(relation) = row.relation.as_deref() else {
            return;
        };
        self.relations.extend([relation]);
        if !row.has_predicate {
            self.all_rows.insert(relation);
        } else if let Some((column, key)) = rows::proven_key(row, call) {
            self.columns.entry(relation).or_default().insert(column);
            self.rows.entry((relation, column)).or_default().insert(key);
        } else {
            self.unknown_rows.insert(relation);
        }
    }

    pub(super) fn matches(&self, relation: &str, row: &RowAccess, call: &EmbeddedSqlCall) -> bool {
        self.all_rows.contains(relation)
            || (!row.has_predicate && self.relations.contains(relation))
            || rows::proven_key(row, call).is_some_and(|(column, key)| {
                self.rows
                    .get(&(relation, column))
                    .is_some_and(|keys| keys.contains(&key))
            })
    }

    pub(super) fn uncertain(
        &self,
        relation: &str,
        row: &RowAccess,
        call: &EmbeddedSqlCall,
    ) -> bool {
        self.relations.contains(relation)
            && (self.unknown_rows.contains(relation)
                || rows::proven_key(row, call).is_none_or(|(column, _)| {
                    self.columns
                        .get(relation)
                        .is_some_and(|columns| columns.len() > 1 || !columns.contains(column))
                }))
    }
}

type Position = (u32, usize);
type Key<'a> = (Transaction, &'a str, &'a str, String);

#[derive(Default)]
pub(super) struct EarlierUpdates<'a> {
    rows: BTreeMap<Key<'a>, Position>,
}

impl<'a> EarlierUpdates<'a> {
    pub(super) fn collect(
        ordered: &[(Transaction, &'a EmbeddedSqlCall, &'a SqlStatementFileFacts)],
    ) -> Self {
        let mut index = Self::default();
        for &(transaction, call, facts) in ordered {
            let access = &facts.lock_writes;
            for (ordinal, statement) in access.share_statements.iter().enumerate() {
                for lock in &statement.locks {
                    if let (SequenceMode::Update, Some(row)) = (lock.mode, lock.row) {
                        if let Some(key) = Self::key(transaction, call, &access.locks[row]) {
                            index
                                .rows
                                .entry(key)
                                .or_insert((call.execution_range.1, ordinal));
                        }
                    }
                }
            }
        }
        index
    }

    pub(super) fn contains(
        &self,
        transaction: Transaction,
        call: &EmbeddedSqlCall,
        ordinal: usize,
        row: &'a RowAccess,
    ) -> bool {
        Self::key(transaction, call, row)
            .and_then(|key| self.rows.get(&key))
            .is_some_and(|position| *position < (call.execution_range.1, ordinal))
    }

    fn key(
        transaction: Transaction,
        call: &EmbeddedSqlCall,
        row: &'a RowAccess,
    ) -> Option<Key<'a>> {
        let (column, key) = rows::proven_key(row, call)?;
        Some((transaction, row.relation.as_deref()?, column, key))
    }
}
