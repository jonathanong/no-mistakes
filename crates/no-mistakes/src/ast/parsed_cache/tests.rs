use super::ParsedProgramCache;

pub(in crate::ast) fn len(cache: &ParsedProgramCache) -> usize {
    cache.entries.borrow().len()
}
