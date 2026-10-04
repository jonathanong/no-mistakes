use super::super::locations::Locations;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Location, Span, Tokenizer};

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

#[test]
fn source_tokens_keep_unicode_byte_offsets_and_character_columns() {
    let sql = fixture("postgres-facts/source/unicode-locations.sql");
    let sql = std::fs::read_to_string(sql).unwrap();
    let locations = Locations::new(&sql);
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
        .tokenize_with_location()
        .unwrap();
    for token in tokens {
        let span = locations.span(token.span).unwrap();
        assert_eq!(locations.range(span.start.offset, span.end.offset), span);
        assert!(sql.is_char_boundary(span.start.offset));
        assert!(sql.is_char_boundary(span.end.offset));
    }
    let name = sql.find('名').unwrap();
    let span = locations.range(name, name + '名'.len_utf8());
    assert_eq!(&sql[span.start.offset..span.end.offset], "名");
    assert_eq!(span.end.column - span.start.column, 1);
    let crab = sql.find('🦀').unwrap();
    assert_eq!(locations.range(crab + 1, crab + 3).start.offset, crab);
    assert_eq!(locations.range(crab + 1, crab + 3).end.offset, crab);
    assert_eq!(locations.range(sql.len(), usize::MAX).end.offset, sql.len());
    assert_eq!(locations.range(name + 3, name).end.offset, name + 3);
}

#[test]
fn absent_and_reversed_locations_have_no_span() {
    let sql =
        std::fs::read_to_string(fixture("postgres-facts/source/unicode-locations.sql")).unwrap();
    let locations = Locations::new(&sql);
    for location in [
        Location::empty(),
        Location::new(1, 0),
        Location::new(999, 1),
        Location::new(1, 999),
        Location::new(u64::MAX, u64::MAX),
    ] {
        assert!(locations.position(location).is_none());
    }
    assert!(locations.span(Span::empty()).is_none());
    assert!(locations
        .span(Span::new(Location::new(1, 2), Location::empty()))
        .is_none());
    assert!(locations
        .span(Span::new(Location::new(1, 2), Location::new(1, 1)))
        .is_none());
}

#[test]
fn empty_and_crlf_sources_have_valid_end_positions() {
    for name in ["empty.sql", "unicode-locations-crlf.sql"] {
        let sql =
            std::fs::read_to_string(fixture(&format!("postgres-facts/source/{name}"))).unwrap();
        let locations = Locations::new(&sql);
        let end = locations.range(sql.len(), sql.len());
        assert_eq!(end.start, end.end);
        assert_eq!(end.start.offset, sql.len());
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
            .tokenize_with_location()
            .unwrap();
        for token in tokens {
            assert!(locations.span(token.span).is_some());
        }
    }
}
