use super::LookupHistory;

impl LookupHistory {
    pub(in crate::statements::keyed_lookups) fn with_guards(
        &mut self,
        guards: &Self,
        inspect: impl FnOnce(&mut Self),
    ) {
        let previous = std::mem::replace(&mut self.active, guards.active.clone());
        inspect(self);
        self.active = previous;
    }

    pub(in crate::statements::keyed_lookups) fn permit_alternative(&mut self, other: &Self) {
        self.active.columns.retain(|column, allowed| {
            let Some(alternative) = other.active.columns.get(column) else {
                return false;
            };
            *allowed |= alternative;
            true
        });
    }
}
