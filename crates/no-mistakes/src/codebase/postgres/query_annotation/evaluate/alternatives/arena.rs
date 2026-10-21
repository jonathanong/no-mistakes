use super::super::Value;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;
use crate::fx::FxHashMap;
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(super) struct Arena<'a> {
    pub objects: &'a ArgumentObjects,
    pub extras: &'a FxHashMap<u64, BTreeMap<usize, Value>>,
}

pub(super) struct ArenaMut<'a> {
    pub objects: &'a mut ArgumentObjects,
    pub extras: &'a mut FxHashMap<u64, BTreeMap<usize, Value>>,
}
impl ArenaMut<'_> {
    pub fn read(&self) -> Arena<'_> {
        Arena {
            objects: self.objects,
            extras: self.extras,
        }
    }
}
