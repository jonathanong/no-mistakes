use crate::codebase::postgres::{
    CatalogCoverage, CatalogTriggerWrite, CatalogTriggerWrites, SchemaCatalog, TriggerEvent,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) struct Triggers<'a> {
    catalog: Option<&'a SchemaCatalog>,
    sources: BTreeMap<&'a str, Vec<&'a CatalogTriggerWrites>>,
}
impl<'a> Triggers<'a> {
    pub fn new(catalog: Option<&'a SchemaCatalog>) -> Self {
        let effects = catalog
            .filter(|catalog| catalog.coverage() == CatalogCoverage::Complete)
            .map(|catalog| catalog.trigger_write_effects())
            .unwrap_or_default();
        let mut sources: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for effect in effects {
            sources
                .entry(effect.source_relation.as_str())
                .or_default()
                .push(effect);
        }
        Self { catalog, sources }
    }
    pub fn canonical<'b>(&'b self, relation: &'b str) -> &'b str {
        self.catalog
            .and_then(|catalog| catalog.relation(relation))
            .map(|table| table.name.as_str())
            .unwrap_or(relation)
    }
    pub fn relations(&self, writes: &[CatalogTriggerWrite]) -> BTreeSet<String> {
        let mut relations = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut pending: VecDeque<_> = writes.iter().map(|write| (write, 0)).collect();
        while let Some((write, depth)) = pending.pop_front() {
            if depth >= 32
                || !seen.insert((
                    write.relation.as_str(),
                    write.event as u8,
                    write.columns.clone(),
                ))
            {
                continue;
            }
            for trigger in self
                .sources
                .get(self.canonical(&write.relation))
                .into_iter()
                .flatten()
            {
                if !trigger.events.contains(&write.event)
                    || (write.event == TriggerEvent::Update
                        && !trigger.update_columns.is_empty()
                        && !trigger
                            .update_columns
                            .iter()
                            .any(|column| write.columns.contains(column)))
                {
                    continue;
                }
                for target in &trigger.writes {
                    relations.insert(self.canonical(&target.relation).to_owned());
                    pending.push_back((target, depth + 1));
                }
            }
        }
        relations
    }
}
