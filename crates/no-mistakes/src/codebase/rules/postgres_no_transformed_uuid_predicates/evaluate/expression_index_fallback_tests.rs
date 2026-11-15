use super::{catalog_tokens, tokens};

#[test]
fn catalog_tokens_keep_source_when_normalization_is_not_parentheses() {
    let unparsed = "lower(id::text";
    assert_eq!(catalog_tokens(unparsed), tokens(unparsed));

    let printed = "lower(id NOTNULL)";
    let kept = catalog_tokens(printed).expect("source tokens");
    assert_eq!(kept, tokens(printed).expect("raw tokens"));
    let displayed = crate::codebase::postgres::parse_postgres_expression(printed)
        .expect("parsed")
        .to_string();
    assert_ne!(kept, tokens(&displayed).expect("printed tokens"));

    assert_eq!(
        catalog_tokens("lower((id)::text)"),
        tokens("lower(id::text)")
    );
}
