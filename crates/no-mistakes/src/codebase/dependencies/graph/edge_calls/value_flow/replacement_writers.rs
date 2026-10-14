use super::*;

#[derive(Default)]
struct WriterFiles<'a> {
    exact: FxHashMap<String, (&'a std::path::Path, bool)>,
    dynamic: Option<(&'a std::path::Path, bool)>,
}
#[derive(Default)]
pub(super) struct ForeignWriters<'a> {
    receivers: FxHashMap<usize, WriterFiles<'a>>,
}
impl<'a> ForeignWriters<'a> {
    pub(super) fn add(
        &mut self,
        path: &'a std::path::Path,
        file: &CallableFileIndex,
        receivers: &FxHashMap<String, usize>,
        replacements: &crate::codebase::dependencies::extract::ValueReplacementFacts,
    ) {
        for write in &replacements.writes {
            if let Some(receiver) = receivers.get(&write.receiver) {
                self.record(*receiver, write.property.as_deref(), path);
            }
        }
        // Assignment summaries contain omitted local aliases too. Project them
        // once while the normalized receiver identities are already available.
        if let Some(names) = file
            .values
            .receiver_mutations
            .as_ref()
            .and_then(|histories| histories.get(&0))
        {
            for (name, receiver) in receivers {
                let Some(history) = names.get(name) else {
                    continue;
                };
                if let Some(assigned) = &history.assigned {
                    for property in assigned.members.keys() {
                        self.record(*receiver, Some(property), path);
                    }
                    if assigned.dynamic.is_some() {
                        self.record(*receiver, None, path);
                    }
                }
                if history
                    .dynamic
                    .as_ref()
                    .is_some_and(|history| history.summary.is_some())
                {
                    self.record(*receiver, None, path);
                }
            }
        }
    }
    fn record(&mut self, receiver: usize, property: Option<&str>, path: &'a std::path::Path) {
        let writers = self.receivers.entry(receiver).or_default();
        let entry = match property {
            Some(property) => writers
                .exact
                .entry(property.to_owned())
                .or_insert((path, false)),
            None => writers.dynamic.get_or_insert((path, false)),
        };
        entry.1 |= entry.0 != path;
    }
    pub(super) fn summary(&self, receiver: usize) -> Option<(FxHashSet<String>, bool)> {
        let writers = self.receivers.get(&receiver)?;
        Some((
            writers.exact.keys().cloned().collect(),
            writers.dynamic.is_some(),
        ))
    }
    pub(super) fn contains(&self, receiver: usize, property: &str, path: &std::path::Path) -> bool {
        self.receivers.get(&receiver).is_some_and(|writers| {
            [writers.exact.get(property), writers.dynamic.as_ref()]
                .into_iter()
                .flatten()
                .any(|(writer, multiple)| *multiple || *writer != path)
        })
    }
}
