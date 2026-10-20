use super::rewrite;
use crate::codebase::postgres::embedded::placeholders::{
    publish_placeholders, INTERNAL_PLACEHOLDER,
};

#[test]
fn aligned_origins_expand_with_the_published_marker() {
    let sql = format!("A{INTERNAL_PLACEHOLDER}1B");
    let origins: Vec<u32> = (0..sql.len() as u32).collect();
    let out = rewrite(&sql, &origins, 0, true);
    assert_eq!(out.len(), publish_placeholders(sql).len());
}

#[test]
fn short_origins_do_not_panic() {
    // `rewrite` asserts equal lengths in debug builds before this fallback.
    // Pin the release vectors here: one origin cannot cover the prefix, the
    // marker, or the tail, and an empty list drops every span.
    let prefixed = format!("SELECT {INTERNAL_PLACEHOLDER}1");
    assert_eq!(
        super::rewrite_impl(&prefixed, &[9], 0, true),
        Vec::<u32>::new()
    );

    let placeholder = format!("{INTERNAL_PLACEHOLDER}1 tail");
    assert_eq!(
        super::rewrite_impl(&placeholder, &[], 1, false),
        Vec::<u32>::new()
    );

    if cfg!(debug_assertions) {
        let prefixed_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rewrite(&prefixed, &[9], 0, true);
        }));
        let placeholder_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rewrite(&placeholder, &[], 1, false);
        }));
        assert!(prefixed_panic.is_err());
        assert!(placeholder_panic.is_err());
    }
}
