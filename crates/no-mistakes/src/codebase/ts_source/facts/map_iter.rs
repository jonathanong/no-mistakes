use super::{TsFactMap, TsFactSlot, TsFileFacts};
use std::path::PathBuf;

impl TsFactMap {
    fn materialize_shared(&mut self) {
        for (_, slot) in &mut self.facts {
            if matches!(slot, TsFactSlot::Shared(_)) {
                *slot = TsFactSlot::Owned(Box::new(slot.clone().into_owned()));
            }
        }
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
    type IntoIter = std::vec::IntoIter<(&'a PathBuf, &'a TsFileFacts)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

impl<'a> IntoIterator for &'a mut TsFactMap {
    type Item = (&'a PathBuf, &'a mut TsFileFacts);
    type IntoIter = std::vec::IntoIter<(&'a PathBuf, &'a mut TsFileFacts)>;

    fn into_iter(self) -> Self::IntoIter {
        self.materialize_shared();
        (&mut self.facts)
            .into_iter()
            .map(|(path, slot)| match slot {
                TsFactSlot::Owned(facts) => (path, facts.as_mut()),
                TsFactSlot::Shared(_) => unreachable!("shared slots were materialized"),
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
