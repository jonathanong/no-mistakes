use super::rewrite;
use crate::codebase::postgres::embedded::placeholders::{
    publish_placeholders, renumber_placeholders, INTERNAL_PLACEHOLDER,
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
        super::rewrite_impl(&prefixed, &[9], 0.into(), true),
        Vec::<u32>::new()
    );

    let placeholder = format!("{INTERNAL_PLACEHOLDER}1 tail");
    assert_eq!(
        super::rewrite_impl(&placeholder, &[], 1.into(), false),
        Vec::<u32>::new()
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "assertion")]
fn rewrite_asserts_on_short_prefixed_origins() {
    let prefixed = format!("SELECT {INTERNAL_PLACEHOLDER}1");
    rewrite(&prefixed, &[9], 0, true);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "assertion")]
fn rewrite_asserts_on_empty_origins() {
    let placeholder = format!("{INTERNAL_PLACEHOLDER}1 tail");
    rewrite(&placeholder, &[], 1, false);
}

#[test]
fn bind_origin_rewriting_tracks_published_and_renumbered_widths() {
    let sql = format!(
        "SELECT {}, {}",
        crate::codebase::postgres::embedded::placeholders::internal_bind(1),
        crate::codebase::postgres::embedded::placeholders::internal_bind(2)
    );
    let origins = vec![13; sql.len()];
    let shifted = renumber_placeholders(&sql, 9);
    let shifted_origins = super::rewrite(&sql, &origins, 9, false);
    assert_eq!(shifted_origins.len(), shifted.len());
    let published = super::rewrite(&shifted, &shifted_origins, 0, true);
    assert_eq!(published.len(), publish_placeholders(shifted).len());
}
