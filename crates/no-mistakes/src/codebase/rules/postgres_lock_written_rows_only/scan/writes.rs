use crate::codebase::postgres::{
    idents::relation_suffix_key, statements::lock_writes::RowAccess, EmbeddedSqlCall,
};
use std::collections::{BTreeMap, BTreeSet};

type WriteRef<'a> = (&'a RowAccess, &'a EmbeddedSqlCall);
#[derive(Default)]
pub(super) struct Writes<'a> {
    relations: BTreeMap<u32, BTreeMap<&'a str, Vec<WriteRef<'a>>>>,
    suffixes: BTreeMap<u32, BTreeMap<&'a str, BTreeSet<&'a str>>>,
}
impl<'a> Writes<'a> {
    pub fn insert(&mut self, transaction: u32, write: &'a RowAccess, call: &'a EmbeddedSqlCall) {
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
