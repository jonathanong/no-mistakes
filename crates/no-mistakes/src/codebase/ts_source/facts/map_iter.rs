use super::{TsFactMap, TsFactSlot, TsFileFacts};
use crate::codebase::ts_source::{FileIdMapIter, FileIdMapIterMut};
use std::path::PathBuf;

impl TsFactMap {
    fn materialize_shared(&mut self) {
        for (_, slot) in &mut self.facts {
            slot.materialize_owned();
        }
    }
}

#[doc(hidden)]
pub struct TsFactMapIter<'a> {
    inner: FileIdMapIter<'a, TsFactSlot>,
}

impl<'a> Iterator for TsFactMapIter<'a> {
    type Item = (&'a PathBuf, &'a TsFileFacts);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(path, slot)| (path, slot.as_facts()))
    }
}

#[doc(hidden)]
pub struct TsFactMapIterMut<'a> {
    inner: FileIdMapIterMut<'a, TsFactSlot>,
}

impl<'a> Iterator for TsFactMapIterMut<'a> {
    type Item = (&'a PathBuf, &'a mut TsFileFacts);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(path, slot)| (path, slot.as_facts_mut()))
    }
}

fn owned_fact_entry((path, slot): (PathBuf, TsFactSlot)) -> (PathBuf, TsFileFacts) {
    (path, slot.into_owned())
}

#[doc(hidden)]
pub struct TsFactMapIntoIter {
    inner: std::vec::IntoIter<(PathBuf, TsFactSlot)>,
}

impl Iterator for TsFactMapIntoIter {
    type Item = (PathBuf, TsFileFacts);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(owned_fact_entry)
    }
}

impl IntoIterator for TsFactMap {
    type Item = (PathBuf, TsFileFacts);
    type IntoIter = TsFactMapIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        TsFactMapIntoIter {
            inner: self.facts.into_entries(),
        }
    }
}

impl<'a> IntoIterator for &'a TsFactMap {
    type Item = (&'a PathBuf, &'a TsFileFacts);
    type IntoIter = TsFactMapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TsFactMapIter {
            inner: self.facts.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a mut TsFactMap {
    type Item = (&'a PathBuf, &'a mut TsFileFacts);
    type IntoIter = TsFactMapIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.bump_playwright_scan_generation();
        self.materialize_shared();
        TsFactMapIterMut {
            inner: (&mut self.facts).into_iter(),
        }
    }
}
