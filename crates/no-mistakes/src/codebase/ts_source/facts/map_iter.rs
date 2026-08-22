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

impl IntoIterator for TsFactMap {
    type Item = (PathBuf, TsFileFacts);
    type IntoIter = std::vec::IntoIter<(PathBuf, TsFileFacts)>;

    fn into_iter(self) -> Self::IntoIter {
        self.facts
            .into_iter()
            .map(|(path, slot)| (path, slot.into_owned()))
            .collect::<Vec<_>>()
            .into_iter()
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
        self.materialize_shared();
        TsFactMapIterMut {
            inner: (&mut self.facts).into_iter(),
        }
    }
}
