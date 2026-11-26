use super::file_diff::FileDiff;
use super::{CategoryChanges, ChangesReport};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) struct ReportBuilder<'a> {
    report: ChangesReport,
    categories: BTreeMap<&'a str, CategoryBuilder>,
    deferred: Vec<(&'a str, String)>,
}

#[derive(Default)]
struct CategoryBuilder {
    keys: BTreeMap<String, Presence>,
}

#[derive(Default)]
struct Presence {
    touched: bool,
    before: usize,
    after: usize,
    attribute_paths: Option<Arc<[String]>>,
}

#[derive(Clone)]
pub(super) struct Observation {
    pub touched: Vec<String>,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub attribute_paths: Option<Arc<[String]>>,
}

impl<'a> ReportBuilder<'a> {
    pub fn new(names: impl IntoIterator<Item = &'a String>, reasons: Vec<String>) -> Self {
        Self {
            report: ChangesReport::new(reasons),
            categories: names
                .into_iter()
                .map(|name| (name.as_str(), CategoryBuilder::default()))
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
        self.report.changed_files.extend(diff.changed_file);
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

impl CategoryBuilder {
    fn record(&mut self, observation: Observation) {
        for id in observation.touched {
            let presence = self.keys.entry(id).or_default();
            presence.touched = true;
            presence
                .attribute_paths
                .clone_from(&observation.attribute_paths);
        }
        for id in observation.before {
            self.keys.entry(id).or_default().before += 1;
        }
        for id in observation.after {
            self.keys.entry(id).or_default().after += 1;
        }
    }

    fn finish(self) -> CategoryChanges {
        let mut changes = CategoryChanges::default();
        for (id, presence) in self.keys {
            if !presence.touched {
                continue;
            }
            if presence.before > 0 && presence.after > 0 {
                if let (1, 1, Some(paths)) =
                    (presence.before, presence.after, presence.attribute_paths)
                {
                    changes.attribute_paths.insert(id.clone(), paths.to_vec());
                }
                changes.changed.push(id);
            } else if presence.after > 0 {
                changes.added.push(id);
            } else {
                changes.removed.push(id);
            }
        }
        changes
    }
}

pub(super) fn path_change(
    id: String,
    old: Option<&[u8]>,
    new: Option<&[u8]>,
    unchanged: bool,
    attribute_paths: Option<Arc<[String]>>,
) -> Observation {
    Observation {
        before: old.map(|_| id.clone()).into_iter().collect(),
        after: new.map(|_| id.clone()).into_iter().collect(),
        touched: (!unchanged).then_some(id).into_iter().collect(),
        attribute_paths,
    }
}
