use crate::codebase::postgres::idents::relation_suffix_key;
use std::collections::BTreeSet;

pub(super) fn same(left: Option<&str>, right: Option<&str>) -> Option<bool> {
    let (left, right) = (left?, right?);
    let left_leaf = relation_suffix_key(left);
    let right_leaf = relation_suffix_key(right);
    if left_leaf == right_leaf && ((left == left_leaf) != (right == right_leaf)) {
        None
    } else {
        Some(left == right)
    }
}

#[derive(Default)]
pub(super) struct Writes<'a> {
    exact: BTreeSet<&'a str>,
    qualified_leaves: BTreeSet<&'a str>,
}

impl<'a> Writes<'a> {
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }

    pub fn clear(&mut self) {
        self.exact.clear();
        self.qualified_leaves.clear();
    }

    pub fn extend(&mut self, writes: impl IntoIterator<Item = &'a str>) {
        for relation in writes {
            self.exact.insert(relation);
            let leaf = relation_suffix_key(relation);
            if relation != leaf {
                self.qualified_leaves.insert(leaf);
            }
        }
    }

    pub fn contains(&self, relation: &str) -> bool {
        self.exact.contains(relation)
    }

    pub fn ambiguous(&self, relation: &str) -> bool {
        let leaf = relation_suffix_key(relation);
        if relation == leaf {
            self.qualified_leaves.contains(leaf)
        } else {
            self.exact.contains(leaf)
        }
    }
}
