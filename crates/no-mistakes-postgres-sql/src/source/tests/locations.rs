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
fn span_covering_grows_only_for_one_adjacent_rendered_alignment() {
    let sql = "-(1) + now()";
    let locations = Locations::new(sql);
    assert!(locations.span_covering(None, "-(1)").is_none());
    let full = locations.range(0, sql.len());
    assert_eq!(
        locations.span_covering(Some(full.clone()), sql).as_ref(),
        Some(&full)
    );
    let signed = locations
        .span_covering(Some(locations.range(2, 3)), "-(1)")
        .unwrap();
    assert_eq!(&sql[signed.start.offset..signed.end.offset], "-(1)");
    let call = locations
        .span_covering(Some(locations.range(7, 10)), "now()")
        .unwrap();
    assert_eq!(&sql[call.start.offset..call.end.offset], "now()");
    let spaced = Locations::new("now ()");
    assert_eq!(
        spaced.slice(
            &spaced
                .span_covering(Some(spaced.range(0, 3)), "now()")
                .unwrap()
        ),
        "now"
    );
    let empty = locations.range(2, 2);
    assert_eq!(
        locations.span_covering(Some(empty.clone()), "1").as_ref(),
        Some(&empty)
    );
    // `aaa` fits the middle byte of `aaaa` in two places, so the span stays put.
    let repeated = Locations::new("aaaa");
    let middle = repeated.range(1, 2);
    assert_eq!(
        repeated.span_covering(Some(middle.clone()), "aaa").as_ref(),
        Some(&middle)
    );
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
