use super::{
    apply_stored_generated_mode, has_non_ws, keyword_of, skip_balanced_parens,
    skip_generated_always_as_expr, split_statement_tokens, stored_token,
};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Word};

fn word(keyword: Keyword, value: &str) -> Token {
    Token::Word(Word {
        value: value.to_string(),
        quote_style: None,
        keyword,
    })
}

#[test]
fn helpers_cover_non_keyword_and_empty_splits() {
    assert!(keyword_of(&Token::SemiColon).is_none());
    assert!(!has_non_ws(&[Token::Whitespace(
        sqlparser::tokenizer::Whitespace::Space
    )]));
    assert!(split_statement_tokens(vec![
        Token::SemiColon,
        Token::Whitespace(sqlparser::tokenizer::Whitespace::Space),
        Token::SemiColon,
    ])
    .is_empty());
    assert!(skip_generated_always_as_expr(&[word(Keyword::GENERATED, "GENERATED")], 0).is_none());
    assert!(skip_generated_always_as_expr(
        &[
            word(Keyword::GENERATED, "GENERATED"),
            word(Keyword::ALWAYS, "ALWAYS")
        ],
        0
    )
    .is_none());
    assert!(skip_generated_always_as_expr(
        &[
            word(Keyword::GENERATED, "GENERATED"),
            word(Keyword::ALWAYS, "ALWAYS"),
            word(Keyword::AS, "AS"),
            word(Keyword::IDENTITY, "IDENTITY"),
        ],
        0
    )
    .is_none());
    assert!(skip_generated_always_as_expr(
        &[
            word(Keyword::GENERATED, "GENERATED"),
            word(Keyword::NoKeyword, "id"),
        ],
        0
    )
    .is_none());
    assert!(skip_generated_always_as_expr(
        &[
            word(Keyword::GENERATED, "GENERATED"),
            Token::Whitespace(sqlparser::tokenizer::Whitespace::Space),
            word(Keyword::ALWAYS, "ALWAYS"),
            word(Keyword::AS, "AS"),
        ],
        0
    )
    .is_none());
    assert_eq!(
        skip_balanced_parens(
            &[Token::LParen, word(Keyword::NoKeyword, "id"), Token::RParen],
            0
        ),
        Some(3)
    );
    assert!(skip_balanced_parens(&[Token::LParen], 0).is_none());
    let mut tokens = vec![stored_token()];
    apply_stored_generated_mode(&mut tokens, 1);
    assert_eq!(tokens.len(), 2);
    apply_stored_generated_mode(&mut tokens, 0);
    assert_eq!(keyword_of(&tokens[0]), Some(Keyword::STORED));
    let mut insert_at_comma = vec![Token::Comma];
    apply_stored_generated_mode(&mut insert_at_comma, 0);
    assert!(matches!(insert_at_comma[0], Token::Word(_)));
    super::set_stored(&mut Token::SemiColon);
}
