use super::*;
use crate::codebase::postgres::statements::{SqlFactPosition, SqlVariantLocations};

#[test]
fn source_columns_use_owned_foreign_text_and_propagate_missing_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres-facts/grouping/fixture");
    let foreign = root.join("exact-origins.ts");
    let path = root.join("executed.sql");
    let sources = crate::codebase::rules::source_store_for_files(&[path.clone(), foreign.clone()]);
    let source = sources.read_path(&path).unwrap();
    let foreign_text = sources.read_path(&foreign).unwrap();
    let offset = foreign_text.find("SELECT DISTINCT").unwrap();
    let expected = foreign_text[..offset]
        .rsplit('\n')
        .next()
        .unwrap()
        .chars()
        .count()
        + 1;
    let site = SqlFactSite::Grouping(0, 0);
    let mut record = SqlStatementFileFacts::default();
    let mut locations = SqlVariantLocations::default();
    locations.positions.insert(
        site.clone(),
        SqlFactPosition {
            sql_line: 1,
            sql_column: 1,
            source_line: 1,
            source_offset: Some(offset),
            source_path: Some(foreign),
        },
    );
    record.variant_locations = Some(locations);
    assert_eq!(
        label(&record, site.clone(), 7, &path, &source, &sources).unwrap(),
        format!("source column {expected}")
    );
    let position = record
        .variant_locations
        .as_mut()
        .unwrap()
        .positions
        .get_mut(&site)
        .unwrap();
    position.source_path = Some(root.join("missing-origin.ts"));
    assert!(label(&record, site.clone(), 7, &path, &source, &sources).is_err());
    record
        .variant_locations
        .as_mut()
        .unwrap()
        .positions
        .get_mut(&site)
        .unwrap()
        .source_path = None;
    record
        .variant_locations
        .as_mut()
        .unwrap()
        .positions
        .get_mut(&site)
        .unwrap()
        .source_offset = Some(usize::MAX);
    assert_eq!(
        label(&record, site.clone(), 7, &path, &source, &sources).unwrap(),
        "SQL column 7"
    );
    record.variant_locations = None;
    assert_eq!(
        label(&record, site, 7, &path, &source, &sources).unwrap(),
        "SQL column 7"
    );
}
