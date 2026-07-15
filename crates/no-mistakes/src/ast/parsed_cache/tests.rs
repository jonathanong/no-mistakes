use super::ParsedProgramCache;
use std::path::Path;

pub(in crate::ast) fn len(cache: &ParsedProgramCache) -> usize {
    cache.entries.borrow().len()
}

#[test]
fn cached_parse_errors_are_available_without_reparsing() {
    let cache = ParsedProgramCache::default();
    let path = Path::new("unsupported.runner-config");

    let error = cache.with_program(path, "", |_, _| ()).unwrap_err();

    assert_eq!(cache.parse_error(path).as_deref(), Some(error.as_str()));
    assert!(cache.parse_error(Path::new("not-cached.ts")).is_none());
}
