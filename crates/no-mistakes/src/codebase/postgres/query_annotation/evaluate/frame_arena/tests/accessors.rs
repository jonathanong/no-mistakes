use super::ScopeArena;
impl ScopeArena {
    /// Live frame count, never a bound on generation-tagged handles.
    pub(in super::super) fn len(&self) -> usize {
        self.ids().count()
    }
}

impl PartialEq for ScopeArena {
    fn eq(&self, other: &Self) -> bool {
        self.ids().count() == other.ids().count()
            && self.ids().all(|env| self.get(env) == other.get(env))
    }
}
impl Eq for ScopeArena {}
