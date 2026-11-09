use super::*;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/postgres-facts/grouping/fixture/executed.sql"
));

#[test]
fn multiline_origins_must_agree_with_legacy_line_ownership() {
    let origins: Vec<u32> = (0..SOURCE.len() as u32).collect();
    assert!(compatible(SOURCE, SOURCE, &origins, 1, &[]));
    let position = EmbeddedSqlSourcePosition {
        sql_line: 1,
        sql_column: 1,
        source_line: 1,
        source_path: None,
    };
    assert!(compatible(
        SOURCE,
        SOURCE,
        &origins,
        80,
        std::slice::from_ref(&position)
    ));
    let shifted = EmbeddedSqlSourcePosition {
        source_line: 2,
        ..position
    };
    assert!(!compatible(SOURCE, SOURCE, &origins, 1, &[shifted]));
    assert!(!compatible(SOURCE, SOURCE, &origins[1..], 1, &[]));
    assert!(!compatible(SOURCE, SOURCE, &origins, 2, &[]));
}

#[test]
fn recovered_origins_require_authored_characters_at_valid_byte_offsets() {
    let origins: Vec<u32> = (0..SOURCE.len() as u32).collect();
    assert!(authored_characters(SOURCE, SOURCE, &origins));
    assert!(!authored_characters(SOURCE, SOURCE, &origins[..1]));
    let mut outside = origins.clone();
    outside[0] = u32::MAX;
    assert!(!authored_characters(SOURCE, SOURCE, &outside));
    let mut stamped = origins;
    stamped.fill(0);
    assert!(!authored_characters(SOURCE, SOURCE, &stamped));
}
