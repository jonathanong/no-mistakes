use super::facts;
use crate::codebase::postgres::SqlPinSource;

#[test]
fn wrapped_scalar_arguments_retain_public_owner_and_bound_sources() {
    let facts = facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/wrapped-scalar-reducer-arrays.sql")));
    assert_eq!(facts.len(), 19);
    for index in [0, 1, 2, 3, 11, 12, 13, 14, 15] {
        let SqlPinSource::Array {
            items,
            scalar_columns,
            indexed_columns,
            cast_types,
        } = &facts[index].query.items[0].pins[0].source
        else {
            panic!("wrapped scalar result lost its argument source: {index}");
        };
        assert_eq!(items, if index >= 14 { &[1, 2][..] } else { &[1][..] });
        assert!(scalar_columns.is_empty());
        assert!(indexed_columns.is_empty());
        assert!(cast_types.is_empty());
    }
    // A direct COALESCE leaf still needs its array-valued column checked by the catalog.
    let SqlPinSource::Array { scalar_columns, .. } = &facts[10].query.items[0].pins[0].source
    else {
        panic!("missing leaf proof")
    };
    assert_eq!(scalar_columns, &[(1, "account_ids".to_string())]);
}
