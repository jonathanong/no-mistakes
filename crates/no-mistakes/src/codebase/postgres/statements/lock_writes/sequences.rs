use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShareStatement {
    pub locks: Vec<Option<String>>,
    pub writes: Range<usize>,
}

pub(super) fn push_share_statement(
    out: &mut super::LockWriteFacts,
    locks: Vec<Option<String>>,
    first_write: usize,
) {
    let writes = first_write..out.writes.len();
    if !locks.is_empty() || !writes.is_empty() {
        out.share_statements.push(ShareStatement { locks, writes });
    }
}
