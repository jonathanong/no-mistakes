use super::TsFileFacts;
use std::sync::Arc;

#[derive(Clone)]
pub(super) enum TsFactSlot {
    Owned(Box<TsFileFacts>),
    Shared(Arc<TsFileFacts>),
}

impl TsFactSlot {
    pub(super) fn as_facts(&self) -> &TsFileFacts {
        match self {
            Self::Owned(facts) => facts,
            Self::Shared(facts) => facts.as_ref(),
        }
    }

    pub(super) fn into_owned(self) -> TsFileFacts {
        match self {
            Self::Owned(facts) => *facts,
            Self::Shared(facts) => Arc::unwrap_or_clone(facts),
        }
    }

    pub(super) fn materialize_owned(&mut self) {
        if matches!(self, Self::Owned(_)) {
            return;
        }
        // Take the slot first so a uniquely owned Shared Arc can unwrap
        // instead of cloning after a temporary extra strong count.
        let taken = std::mem::replace(self, Self::Owned(Box::default()));
        *self = Self::Owned(Box::new(taken.into_owned()));
    }

    pub(super) fn as_facts_mut(&mut self) -> &mut TsFileFacts {
        match self {
            Self::Owned(facts) => facts,
            Self::Shared(_) => unreachable!("shared slots were materialized"),
        }
    }
}
