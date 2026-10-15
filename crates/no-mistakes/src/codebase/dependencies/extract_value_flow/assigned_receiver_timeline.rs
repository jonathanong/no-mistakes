use super::*;

#[derive(Default)]
pub(super) struct Timeline {
    pub(super) events: std::collections::BTreeMap<u32, Origin>,
    aliases: FxHashMap<(usize, Option<CallableId>), FxHashSet<String>>,
}
impl Timeline {
    pub(super) fn append(&mut self, other: &mut Self) {
        self.events.append(&mut other.events);
        // A component union can interleave independent prefix histories. Only
        // a subsequent exact store establishes a new alias-sharing certificate.
        self.aliases.clear();
    }
    pub(super) fn alias_seen(&self, value: &ValueExpression) -> bool {
        match value {
            ValueExpression::Named {
                name,
                scope,
                context,
                ..
            } => self
                .aliases
                .get(&(scope.unwrap_or(0), *context))
                .is_some_and(|names| names.contains(name)),
            _ => false,
        }
    }
    pub(super) fn remember_alias(&mut self, value: &ValueExpression, complete: bool, stable: bool) {
        if complete {
            self.aliases.clear();
        }
        if stable {
            if let ValueExpression::Named {
                name,
                scope,
                context,
                ..
            } = value
            {
                self.aliases
                    .entry((scope.unwrap_or(0), *context))
                    .or_default()
                    .insert(name.clone());
            }
        }
    }
}
