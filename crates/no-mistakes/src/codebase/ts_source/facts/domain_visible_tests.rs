use super::*;
use std::path::Path;

#[test]
fn empty_visible_set_stays_an_explicit_empty_universe() {
    let mut context = TsFactContext::new(Path::new("/tmp"));
    context.set_visible_file_set(crate::fx::PathSet::default());

    let visible = context
        .visible_files
        .as_ref()
        .expect("empty set stays Some");
    assert!(visible.is_empty());
}
