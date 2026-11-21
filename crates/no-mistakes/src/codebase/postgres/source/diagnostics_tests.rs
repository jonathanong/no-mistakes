use super::super::locations::Locations;
use super::unavailable_boundary;
use sqlparser::tokenizer::Location;

#[test]
fn an_unmappable_boundary_keeps_the_diagnostic_off_the_source_slice() {
    let locations = Locations::new("SELECT id FROM orders");
    let mut diagnostics = Vec::new();
    unavailable_boundary(
        &mut diagnostics,
        &locations,
        Location { line: 1, column: 1 },
    );
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].span.is_some());
    assert!(diagnostics[0]
        .message
        .contains("Statement source boundary is unavailable"));

    unavailable_boundary(
        &mut diagnostics,
        &locations,
        Location { line: 0, column: 0 },
    );
    assert!(diagnostics[1].span.is_none());
}
