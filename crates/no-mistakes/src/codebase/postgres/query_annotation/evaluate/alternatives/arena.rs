use super::super::Value;
use crate::fx::FxHashMap;
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(super) struct Arena<'a> {
    pub objects: &'a FxHashMap<u64, Vec<Value>>,
    pub extras: &'a FxHashMap<u64, BTreeMap<usize, Value>>,
}

pub(super) struct ArenaMut<'a> {
    pub objects: &'a mut FxHashMap<u64, Vec<Value>>,
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
