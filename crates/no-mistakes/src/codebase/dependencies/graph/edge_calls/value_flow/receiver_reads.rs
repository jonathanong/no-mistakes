use super::*;
use default_origins::ReceiverRead;

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
    ) -> Option<u32> {
        self.iter(id)
            .filter(|read| read.file == file && read.context == context)
            .map(|read| read.offset)
            .max()
    }
}
