use super::{unpublish, unpublish_origins};
use crate::codebase::postgres::embedded::{
    placeholders::{internal_bind, internal_placeholder, publish_placeholders_with_positions},
    SqlByteOrigins,
};

#[test]
fn generated_binds_roundtrip_without_rewriting_authored_parameters() {
    let internal = format!(
        "SELECT $9, {}, {}",
        internal_bind(1),
        internal_placeholder(2)
    );
    let (published, positions) = publish_placeholders_with_positions(internal.clone());
    assert_eq!(published, "SELECT $9, $1, sql_placeholder_2");
    assert_eq!(positions, [(1, 12), (1, 16)]);
    assert_eq!(unpublish(&published, &positions), Some(internal.clone()));
    let origins = SqlByteOrigins::from(vec![7; published.len()]);
    assert_eq!(
        unpublish_origins(&published, &positions, &origins),
        Some(vec![7; internal.len()])
    );
}
