use super::rows::compare_keys;
use crate::codebase::postgres::{
    idents::relation_suffix_key,
    statements::lock_writes::{LockWriteFacts, RowAccess},
    EmbeddedSqlCall,
};
use std::collections::{BTreeMap, BTreeSet};

type WriteRef<'a> = (&'a RowAccess, &'a EmbeddedSqlCall);
type TransactionWrites<'a> = BTreeMap<u32, BTreeMap<&'a str, Vec<WriteRef<'a>>>>;
#[derive(Default)]
pub(super) struct Writes<'a> {
    relations: TransactionWrites<'a>,
    upserts: TransactionWrites<'a>,
    suffixes: BTreeMap<u32, BTreeMap<&'a str, BTreeSet<&'a str>>>,
}
impl<'a> Writes<'a> {
    pub fn insert(&mut self, transaction: u32, facts: &'a LockWriteFacts, call: &'a EmbeddedSqlCall) {
        for write in &facts.writes {
            self.insert_write(transaction, write, call);
        }
        for (relation, write) in facts
            .upsert_write_keys
            .iter()
            .filter_map(|write| Some((write.relation.as_deref()?, write)))
        {
            self.upserts
                .entry(transaction)
                .or_default()
                .entry(relation)
                .or_default()
                .push((write, call));
        }
    }

    pub fn matches_upsert(
        &self,
        transaction: u32,
        relation: &str,
        lock: &RowAccess,
        call: &EmbeddedSqlCall,
    ) -> Option<bool> {
        let candidates = self
            .upserts
            .get(&transaction)
            .and_then(|relations| relations.get(relation))
            .filter(|_| lock.has_predicate);
        let mut unknown = false;
        for (write, write_call) in candidates.into_iter().flatten() {
            if lock.key.as_ref().zip(write.key.as_ref()).is_some_and(
                |((lock_column, _), (write_column, _))| lock_column != write_column,
            ) {
                continue;
            }
            match compare_keys(lock, call, write, write_call) {
                Some(true) => return Some(true),
                Some(false) => {}
                None => unknown = true,
            }
        }
        (!unknown).then_some(false)
    }

    fn insert_write(&mut self, transaction: u32, write: &'a RowAccess, call: &'a EmbeddedSqlCall) {
        let Some(relation) = write.relation.as_deref() else {
            return;
        };
        self.relations
            .entry(transaction)
            .or_default()
            .entry(relation)
            .or_default()
            .push((write, call));
        self.suffixes
            .entry(transaction)
            .or_default()
            .entry(relation_suffix_key(relation))
            .or_default()
            .insert(relation);
    }
    pub fn for_relation(&self, transaction: u32, relation: &str) -> (&[WriteRef<'a>], bool) {
        let candidates = self
            .relations
            .get(&transaction)
            .and_then(|relations| relations.get(relation))
            .map(Vec::as_slice)
            .unwrap_or_default();
        let ambiguous = self
            .suffixes
            .get(&transaction)
            .and_then(|suffixes| suffixes.get(relation_suffix_key(relation)))
            .is_some_and(|relations| relations.len() > 1 || !relations.contains(relation));
        (candidates, ambiguous)
    }
}
