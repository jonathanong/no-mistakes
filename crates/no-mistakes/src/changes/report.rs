use super::file_diff::FileDiff;
use super::{ChangesReport, KeyChanges};
use std::collections::BTreeMap;

pub(super) struct ReportBuilder<'a> {
    report: ChangesReport,
    categories: BTreeMap<&'a str, CategoryChanges>,
    deferred: Vec<(&'a str, String)>,
}

#[derive(Default)]
struct CategoryChanges {
    keys: BTreeMap<String, Presence>,
}

#[derive(Default)]
struct Presence {
    touched: bool,
    before: bool,
    after: bool,
}

#[derive(Clone)]
pub(super) struct Observation {
    pub touched: Vec<String>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

impl<'a> ReportBuilder<'a> {
    pub fn new(names: impl IntoIterator<Item = &'a String>, reasons: Vec<String>) -> Self {
        Self {
            report: ChangesReport::new(reasons),
            categories: names
                .into_iter()
                .map(|name| (name.as_str(), CategoryChanges::default()))
                .collect(),
            deferred: Vec::new(),
        }
    }

    pub fn record(&mut self, diff: FileDiff<'a>) {
        for (name, observation) in diff.categories {
            self.categories
                .get_mut(name)
                .expect("compiled category")
                .record(observation);
        }
        self.report.unclassified.extend(diff.unclassified);
        self.report.reasons.extend(diff.reasons);
        self.deferred.extend(diff.deferred);
    }

    pub fn finish(mut self) -> ChangesReport {
        self.report.categories = self
            .categories
            .into_iter()
            .map(|(name, category)| (name.to_owned(), category.finish()))
            .collect();
        for (name, reason) in self.deferred {
            let changes = &self.report.categories[name];
            if !changes.added.is_empty() || !changes.removed.is_empty() {
                self.report.reasons.push(reason);
            }
        }
        self.report.finish()
    }
}

impl CategoryChanges {
    fn record(&mut self, observation: Observation) {
        for id in observation.touched {
            self.keys.entry(id).or_default().touched = true;
        }
        for id in observation.before {
            self.keys.entry(id).or_default().before = true;
        }
        for id in observation.after {
            self.keys.entry(id).or_default().after = true;
        }
    }

    fn finish(self) -> KeyChanges {
        let mut changes = KeyChanges::default();
        for (id, presence) in self.keys {
            if !presence.touched {
                continue;
            }
            if presence.before && presence.after {
                changes.changed.push(id);
            } else if presence.after {
                changes.added.push(id);
            } else {
                changes.removed.push(id);
            }
        }
        changes
    }
}
