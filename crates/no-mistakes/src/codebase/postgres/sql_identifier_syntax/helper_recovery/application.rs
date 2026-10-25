//! Exact rule-local application ownership; planner-global scan IDs differ.
use super::super::{EmbeddedSqlOptions, FileFacts, SqlCall};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ApplicationKey {
    pub profile: EmbeddedSqlOptions,
    pub rule: String,
    pub index: usize,
}
impl ApplicationKey {
    pub fn new(profile: &EmbeddedSqlOptions, rule: &str, index: usize) -> Self {
        Self {
            profile: profile.clone(),
            rule: rule.to_string(),
            index,
        }
    }
}
impl FileFacts {
    pub fn application_keys(&self) -> std::collections::BTreeSet<ApplicationKey> {
        self.low_applications
            .iter()
            .map(|(profile, rule, index)| ApplicationKey::new(profile, rule, *index))
            .collect()
    }
    pub fn includes_application(&self, key: &ApplicationKey) -> bool {
        self.low_applications.iter().any(|(profile, rule, index)| {
            profile == &key.profile && rule == &key.rule && *index == key.index
        })
    }
    pub fn calls_for(&self, key: &ApplicationKey) -> Option<&[SqlCall]> {
        // Application projections refine a prepared canonical profile; they do
        // not replace missing base facts or revive an unregistered owner.
        let base = self
            .profiles
            .iter()
            .find(|(profile, _)| profile == &key.profile)?
            .1
            .as_slice();
        Some(if self.includes_application(key) {
            self.application_calls.get(key).map_or(base, Vec::as_slice)
        } else {
            base
        })
    }
}
