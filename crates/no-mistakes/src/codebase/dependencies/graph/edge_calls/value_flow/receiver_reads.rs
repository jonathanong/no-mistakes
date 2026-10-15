use super::*;
use default_origins::ReceiverRead;

/// A repeated frame is evidence of unknown ordering, not a missing frame.
#[derive(Clone, Copy)]
pub(super) enum ReadOffset {
    Absent,
    Exact(u32),
    Repeated,
}
impl ReadOffset {
    pub(super) fn known(self) -> Option<u32> {
        match self {
            Self::Exact(offset) => Some(offset),
            Self::Absent | Self::Repeated => None,
        }
    }
    pub(super) fn or_absent(self, fallback: impl FnOnce() -> Option<u32>) -> Self {
        if matches!(self, Self::Absent) {
            fallback().map_or(Self::Absent, Self::Exact)
        } else {
            self
        }
    }
}
pub(super) type ReadId = Option<std::num::NonZeroUsize>;
/// Walk-local links retain invocation paths without copying origin subgraphs.
#[derive(Default)]
pub(super) struct ReceiverReads<'a> {
    links: Vec<(&'a ReceiverRead, ReadId)>,
}
impl<'a> ReceiverReads<'a> {
    pub(super) fn push(&mut self, read: &'a ReceiverRead, prior: ReadId) -> ReadId {
        let id = self.links.len();
        self.links.push((read, prior));
        std::num::NonZeroUsize::new(id + 1)
    }
    pub(super) fn source(
        &mut self,
        read: &'a ReceiverRead,
        prior: ReadId,
        acquiring: bool,
    ) -> ReadId {
        if prior.is_none()
            || self
                .iter(prior)
                .any(|invocation| invocation.preserves(&read.file, read.context, acquiring))
        {
            self.push(read, prior)
        } else {
            prior
        }
    }
    fn iter(&self, mut id: ReadId) -> impl Iterator<Item = &'a ReceiverRead> + '_ {
        std::iter::from_fn(move || {
            let (read, prior) = self.links[id?.get() - 1];
            id = prior;
            Some(read)
        })
    }
    pub(super) fn offset(
        &self,
        id: ReadId,
        file: &std::path::Path,
        context: Option<CallableId>,
    ) -> ReadOffset {
        let mut offset = None;
        for read in self
            .iter(id)
            .filter(|read| read.file == file && read.context == context)
        {
            if read.repeated {
                return ReadOffset::Repeated;
            }
            offset = Some(offset.map_or(read.offset, |old: u32| old.max(read.offset)));
        }
        offset.map_or(ReadOffset::Absent, ReadOffset::Exact)
    }
    pub(super) fn default_owner(&self, id: ReadId, file: &std::path::Path) -> Option<CallableId> {
        self.iter(id).find_map(|read| {
            read.owned_context
                .as_deref()
                .and_then(|(owner_file, owner, preserve)| {
                    (owner_file == file && *preserve).then_some(*owner)
                })
        })
    }
}
