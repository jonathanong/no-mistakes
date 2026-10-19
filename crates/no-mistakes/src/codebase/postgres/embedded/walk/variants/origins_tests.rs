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
    let prefixed = format!("SELECT {INTERNAL_PLACEHOLDER}1");
    let out = rewrite(&prefixed, &[9], 0, true);
    assert!(out.len() < prefixed.len());

    let placeholder = format!("{INTERNAL_PLACEHOLDER}1 tail");
    assert!(rewrite(&placeholder, &[], 1, false).is_empty());
}
