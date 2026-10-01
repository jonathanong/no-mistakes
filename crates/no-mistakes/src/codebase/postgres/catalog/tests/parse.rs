use super::super::partition::{parse_partition_key, PartitionParseError};
use super::super::trigger::parse_trigger;
use super::super::PartitionKeyElement;

#[test]
fn trigger_parser_reports_each_malformed_clause() {
    let cases = [
        (
            "TRIGGER t BEFORE INSERT ON t EXECUTE FUNCTION fn()",
            "expected create",
        ),
        (
            "CREATE t BEFORE INSERT ON t EXECUTE FUNCTION fn()",
            "expected trigger",
        ),
        (
            "CREATE TRIGGER t INSERT ON t EXECUTE FUNCTION fn()",
            "expected trigger timing",
        ),
        (
            "CREATE TRIGGER t INSTEAD INSERT ON t EXECUTE FUNCTION fn()",
            "expected of",
        ),
        (
            "CREATE TRIGGER t BEFORE ON t EXECUTE FUNCTION fn()",
            "expected trigger event",
        ),
        (
            "CREATE TRIGGER t BEFORE UPDATE OF , ON t EXECUTE FUNCTION fn()",
            "expected identifier",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT t EXECUTE FUNCTION fn()",
            "expected on",
        ),
        (
            "CREATE TRIGGER \" ON t EXECUTE FUNCTION fn()",
            "unterminated identifier",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t FOR ROW EXECUTE FUNCTION fn()",
            "expected each",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t FOR EACH STATEMENTZ EXECUTE FUNCTION fn()",
            "expected statement",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t WHEN note EXECUTE FUNCTION fn()",
            "expected (",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t WHEN (note EXECUTE FUNCTION fn()",
            "unbalanced WHEN",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t FUNCTION fn()",
            "expected execute",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t EXECUTE fn()",
            "expected function or procedure",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t EXECUTE FUNCTION fn(",
            "expected string",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t EXECUTE FUNCTION fn('x)",
            "unterminated string",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t EXECUTE FUNCTION fn('x' 'y')",
            "expected )",
        ),
        (
            "CREATE TRIGGER t BEFORE INSERT ON t EXECUTE FUNCTION fn() EXTRA",
            "trailing trigger syntax",
        ),
    ];
    for (definition, message) in cases {
        let error = parse_trigger(definition).unwrap_err();
        assert!(error.contains(message), "{definition} => {error}");
    }
}

#[test]
fn partition_parser_rejects_syntax_and_keeps_trailing_quoted_text_as_an_expression() {
    assert!(matches!(
        parse_partition_key(" (id)"),
        Err(PartitionParseError::Syntax)
    ));
    assert!(matches!(
        parse_partition_key("RANGE (id,)"),
        Err(PartitionParseError::Syntax)
    ));
    assert!(matches!(
        parse_partition_key("RANGE ()"),
        Err(PartitionParseError::Syntax)
    ));
    assert!(matches!(
        parse_partition_key("RANGE (id"),
        Err(PartitionParseError::Syntax)
    ));
    let key = parse_partition_key("RANGE (\"Order\"extra)").unwrap();
    assert_eq!(
        key.elements,
        vec![PartitionKeyElement::Expression(
            "\"Order\"extra".to_string()
        )]
    );
    assert!(matches!(
        parse_partition_key("RANGE id"),
        Err(PartitionParseError::Syntax)
    ));
    assert!(matches!(
        parse_partition_key("RANGE (id) extra"),
        Err(PartitionParseError::Syntax)
    ));
    let underscore = parse_partition_key("RANGE (_id)").unwrap();
    assert_eq!(
        underscore.elements,
        vec![PartitionKeyElement::Column("_id".to_string())]
    );
    let digit = parse_partition_key("RANGE ($id)").unwrap();
    assert_eq!(
        digit.elements,
        vec![PartitionKeyElement::Expression("$id".to_string())]
    );
}
