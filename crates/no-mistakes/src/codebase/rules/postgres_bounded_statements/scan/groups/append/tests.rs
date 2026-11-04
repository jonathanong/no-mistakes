use super::{only_appended, span};
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{SqlStatementFileFacts, SqlVariantLocations};

#[test]
fn missing_relation_locations_prevent_the_all_appended_proof() {
    let mut locations = SqlVariantLocations {
        original_call_line: 20,
        ..Default::default()
    };
    let mapped = SqlFactSite::BoundRelation(0, 1);
    locations.insert(SqlFactSite::Origin, 1, 1);
    locations.insert(mapped.clone(), 1, 1);
    locations.positions.get_mut(&mapped).unwrap().source_line = 5;
    locations.bound_tables.push((0, "invoices".into(), mapped));
    let mut file = SqlStatementFileFacts {
        path: "query.ts".into(),
        variant_locations: Some(locations),
        ..Default::default()
    };
    assert_eq!(only_appended(&[span(&file, 0).unwrap()]), Some(20));
    file.variant_locations.as_mut().unwrap().bound_tables.push((
        0,
        "accounts".into(),
        SqlFactSite::BoundRelation(0, 0),
    ));
    let incomplete = span(&file, 0).unwrap();
    assert_eq!(incomplete.relations, [(5, 4)]);
    assert_eq!(only_appended(&[incomplete]), None);
}
