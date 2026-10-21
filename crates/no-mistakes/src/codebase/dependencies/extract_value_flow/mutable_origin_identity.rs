use super::*;

impl RootVersion {
    pub(in super::super) fn known_identity(self) -> Option<RootIdentity> {
        self.selected.map(|_| self.identity())
    }

    pub(in super::super) fn identity(self) -> RootIdentity {
        RootIdentity {
            context: self.context,
            selected: self.selected,
        }
    }

    pub(in super::super) fn same_selected(&self, other: &Self) -> bool {
        self.context == other.context && self.selected == other.selected
    }
}

#[cfg(test)]
#[path = "mutable_origin_identity_tests.rs"]
mod tests;
