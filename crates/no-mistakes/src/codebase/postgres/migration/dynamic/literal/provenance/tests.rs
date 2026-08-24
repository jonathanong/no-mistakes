use super::*;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

fn characters(value: &str) -> Vec<char> {
    value.chars().collect()
}

#[test]
fn measures_every_postgres_escape_shape() {
    assert_eq!(escaped_width(&characters("\\u0041"), 0), 6);
    assert_eq!(escaped_width(&characters("\\U0001F600"), 0), 10);
    assert_eq!(escaped_width(&characters("\\x41"), 0), 4);
    assert_eq!(escaped_width(&characters("\\x4z"), 0), 3);
    assert_eq!(escaped_width(&characters("\\xz"), 0), 2);
    assert_eq!(escaped_width(&characters("\\101"), 0), 4);
    assert_eq!(escaped_width(&characters("\\8"), 0), 2);
    assert_eq!(escaped_width(&characters("\\"), 0), 1);

    assert_eq!(unicode_escape_width(&characters("!!"), 0, '!'), 2);
    assert_eq!(unicode_escape_width(&characters("!+01F600"), 0, '!'), 8);
    assert_eq!(unicode_escape_width(&characters("!D83D!DE00"), 0, '!'), 10);
    assert_eq!(unicode_escape_width(&characters("!0041"), 0, '!'), 5);
}

#[test]
fn maps_plain_raw_escaped_and_unicode_literal_bytes() {
    assert_eq!(
        map_decoded_source_bytes("a''b", "a'b", LiteralEscape::Plain, 3),
        Some(vec![3, 3, 3])
    );
    assert_eq!(
        map_decoded_source_bytes("a\nb", "a\nb", LiteralEscape::Raw, 4),
        Some(vec![4, 4, 5])
    );
    assert_eq!(
        map_decoded_source_bytes("\\x41", "A", LiteralEscape::Escaped, 6),
        Some(vec![6])
    );
    assert_eq!(
        map_decoded_source_bytes("!!", "!", LiteralEscape::Unicode('!'), 7),
        Some(vec![7])
    );

    let tokens = Tokenizer::new(&PostgreSqlDialect {}, "SELECT")
        .tokenize_with_location()
        .expect("tokenize word");
    assert!(literal_source_bytes("SELECT", &tokens[0], "", None).is_none());
}
