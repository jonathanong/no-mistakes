use super::expect_end;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::{Parser, ParserError};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn validate(code: &[TokenWithSpan]) -> bool {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(code.to_vec());
    parse(&mut parser)
        .and_then(|()| expect_end(&mut parser))
        .is_ok()
}

fn parse(parser: &mut Parser<'_>) -> Result<(), ParserError> {
    if parser.parse_keywords(&[Keyword::COMMENT, Keyword::ON, Keyword::TRIGGER]) {
        parser.parse_identifier()?;
        parser.expect_keyword_is(Keyword::ON)?;
        parser.parse_object_name(false)?;
        parser.expect_keyword_is(Keyword::IS)?;
        if !parser.parse_keyword(Keyword::NULL) {
            parser.parse_literal_string()?;
        }
    } else if parser.parse_keywords(&[Keyword::ALTER, Keyword::INDEX]) {
        let _ = parser.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
        parser.parse_object_name(false)?;
        parser.expect_keywords(&[Keyword::ATTACH, Keyword::PARTITION])?;
        parser.parse_object_name(false)?;
    } else {
        parser.expect_keywords(&[
            Keyword::ALTER,
            Keyword::TEXT,
            Keyword::SEARCH,
            Keyword::CONFIGURATION,
        ])?;
        parser.parse_object_name(false)?;
        mappings(parser)?;
    }
    Ok(())
}

fn mappings(parser: &mut Parser<'_>) -> Result<(), ParserError> {
    if parser.parse_keywords(&[Keyword::ADD]) {
        mapping(parser)?;
        parser.expect_keyword_is(Keyword::FOR)?;
        parser.parse_comma_separated(Parser::parse_identifier)?;
        parser.expect_keyword_is(Keyword::WITH)?;
        parser.parse_comma_separated(|parser| parser.parse_object_name(false))?;
    } else if parser.parse_keywords(&[Keyword::DROP]) {
        mapping(parser)?;
        let _ = parser.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
        parser.expect_keyword_is(Keyword::FOR)?;
        parser.parse_comma_separated(Parser::parse_identifier)?;
    } else {
        parser.expect_keywords(&[Keyword::ALTER])?;
        mapping(parser)?;
        if parser.parse_keyword(Keyword::FOR) {
            parser.parse_comma_separated(Parser::parse_identifier)?;
            if parser.parse_keyword(Keyword::WITH) {
                parser.parse_comma_separated(|parser| parser.parse_object_name(false))?;
                return Ok(());
            }
        }
        parser.expect_keyword_is(Keyword::REPLACE)?;
        parser.parse_object_name(false)?;
        parser.expect_keyword_is(Keyword::WITH)?;
        parser.parse_object_name(false)?;
    }
    Ok(())
}

fn mapping(parser: &mut Parser<'_>) -> Result<(), ParserError> {
    if matches!(parser.next_token().token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("MAPPING"))
    {
        Ok(())
    } else {
        Err(ParserError::ParserError("Expected MAPPING".into()))
    }
}
