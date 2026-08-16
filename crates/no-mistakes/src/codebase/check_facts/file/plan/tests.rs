use super::ts_source;
use std::sync::Arc;

#[test]
fn ts_source_reuses_the_same_arc_pointer() {
    let source: Arc<str> = Arc::from("export const value = 1;\n");
    let first = ts_source(Some(Arc::clone(&source)));
    let second = ts_source(Some(Arc::clone(&source)));

    assert!(Arc::ptr_eq(first.source.as_ref().unwrap(), &source));
    assert!(Arc::ptr_eq(second.source.as_ref().unwrap(), &source));
    assert!(Arc::ptr_eq(
        first.source.as_ref().unwrap(),
        second.source.as_ref().unwrap(),
    ));
}

#[test]
fn ts_source_preserves_missing_source() {
    assert!(ts_source(None).source.is_none());
}
