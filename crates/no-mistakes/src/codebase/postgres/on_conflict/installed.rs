use std::path::{Path, PathBuf};

use crate::codebase::postgres::statement_facts::SqlTriggerFact;

struct Interval<'a> {
    fact: &'a SqlTriggerFact,
    start: usize,
    end: usize,
}

/// Application-scoped installation intervals, grouped by target table.
/// Queries borrow facts without rebuilding or cloning the trigger catalog.
pub(crate) struct InstalledTriggers<'a> {
    positions: Vec<(PathBuf, (usize, usize))>,
    tables: crate::fx::FxHashMap<String, Vec<Interval<'a>>>,
}

impl<'a> InstalledTriggers<'a> {
    pub(crate) fn new(triggers: &'a [SqlTriggerFact]) -> Self {
        let mut events: Vec<_> = triggers
            .iter()
            .filter(|trigger| !trigger.source_path.as_os_str().is_empty())
            .collect();
        events.sort_by(|left, right| {
            left.source_path
                .cmp(&right.source_path)
                .then((left.line, left.column).cmp(&(right.line, right.column)))
                .then(left.removed.cmp(&right.removed))
        });
        if let Some(observer) = crate::diagnostics::current().filter(|observer| observer.verbose())
        {
            observer.increment("postgres.trigger.timeline_builds", 1);
            observer.increment("postgres.trigger.timeline_events", events.len() as u64);
        }
        let mut prepared = Self {
            positions: Vec::with_capacity(events.len()),
            tables: crate::fx::FxHashMap::default(),
        };
        for fact in triggers
            .iter()
            .filter(|fact| fact.source_path.as_os_str().is_empty() && !fact.removed)
        {
            prepared
                .tables
                .entry(fact.table.to_ascii_lowercase())
                .or_default()
                .push(Interval {
                    fact,
                    start: 0,
                    end: usize::MAX,
                });
        }
        let mut active: crate::fx::FxHashMap<String, crate::fx::FxHashMap<String, usize>> =
            Default::default();
        for (index, event) in events.into_iter().enumerate() {
            let position = index + 1;
            prepared
                .positions
                .push((event.source_path.clone(), (event.line, event.column)));
            let name = event.name.to_ascii_lowercase();
            let table = event.table.to_ascii_lowercase();
            let named = active.entry(name).or_default();
            if event.removed && table.is_empty() {
                for (table, slot) in named.drain() {
                    prepared.tables.get_mut(&table).unwrap()[slot].end = position;
                }
                continue;
            }
            let intervals = prepared.tables.entry(table.clone()).or_default();
            if let Some(slot) = named.remove(&table) {
                intervals[slot].end = position;
            }
            if !event.removed {
                named.insert(table, intervals.len());
                intervals.push(Interval {
                    fact: event,
                    start: position,
                    end: usize::MAX,
                });
            }
        }
        prepared
    }

    pub(crate) fn final_state(&self) -> impl Iterator<Item = &SqlTriggerFact> {
        self.tables
            .values()
            .flatten()
            .filter(|interval| interval.end == usize::MAX)
            .map(|interval| interval.fact)
    }

    pub(crate) fn at(
        &self,
        path: &Path,
        location: (usize, usize),
        table: &str,
    ) -> impl Iterator<Item = &SqlTriggerFact> + Clone {
        // SQL include patterns may select .psql or extensionless migrations.
        // Embedded extraction only produces these JavaScript/TypeScript types.
        let embedded = matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("js" | "ts" | "mjs" | "mts" | "cjs" | "cts" | "tsx" | "jsx")
        );
        let position = if embedded {
            self.positions.len()
        } else {
            self.positions
                .partition_point(|(event_path, event_location)| {
                    event_path.as_path() < path
                        || (event_path == path && *event_location < location)
                })
        };
        self.tables
            .get(&table.to_ascii_lowercase())
            .into_iter()
            .flatten()
            .filter(move |interval| interval.start <= position && position < interval.end)
            .map(|interval| interval.fact)
    }
}
