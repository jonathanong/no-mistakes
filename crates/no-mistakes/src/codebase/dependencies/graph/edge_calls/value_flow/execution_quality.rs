use super::*;

pub(super) type Location = (u32, Option<CallableId>);

/// Repetition preserves availability evidence, but cannot order live writes by
/// one lexical execution. Missing evidence and known repetition stay distinct.
#[derive(Clone, Copy)]
pub(super) enum ExecutionPhase {
    Absent,
    Exact(Location),
    Repeated(Option<Location>),
}
impl ExecutionPhase {
    pub(super) fn repeated(self, repeated: bool) -> Self {
        if repeated {
            Self::Repeated(self.location())
        } else {
            self
        }
    }
    pub(super) fn location(self) -> Option<Location> {
        match self {
            Self::Absent => None,
            Self::Exact(location) => Some(location),
            Self::Repeated(location) => location,
        }
    }
    pub(super) fn is_repeated(self) -> bool {
        matches!(self, Self::Repeated(_))
    }
    pub(super) fn or_absent(self, fallback: Self) -> Self {
        if matches!(self, Self::Absent) {
            fallback
        } else {
            self
        }
    }
}
