//! Recover SQL carried by procedural bodies and concatenated string literals.
use super::super::LocatedStatement;
use super::super::{keyword_of, skip_ws};
use super::locations;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn peel_do_body(tokens: &[Token]) -> Option<String> {
    let mut index = skip_ws(tokens, 0);
    if keyword_of(tokens.get(index)?) != Some(Keyword::DO) {
        return None;
    }
    index = skip_ws(tokens, index + 1);
    let leading_language = keyword_of(tokens.get(index)?) == Some(Keyword::LANGUAGE);
    if leading_language {
        index = skip_ws(tokens, index + 1);
        if !crate::codebase::postgres::parse::is_plpgsql_language(tokens.get(index)?) {
            return None;
        }
        index = skip_ws(tokens, index + 1);
    }
    match tokens.get(index)? {
        Token::DollarQuotedString(body) => {
            let rest = skip_ws(tokens, index + 1);
            if rest >= tokens.len() {
                return Some(body.value.clone());
            }
            if leading_language || keyword_of(tokens.get(rest)?) != Some(Keyword::LANGUAGE) {
                return None;
            }
            let language = skip_ws(tokens, rest + 1);
            if !crate::codebase::postgres::parse::is_plpgsql_language(tokens.get(language)?) {
                return None;
            }
            (skip_ws(tokens, language + 1) >= tokens.len()).then(|| body.value.clone())
        }
        _ => None,
    }
}

pub(super) fn recover_chr_encoded(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Vec<LocatedStatement> {
    let mut rewritten = tokens.to_vec();
    super::super::rewrite_chr_tokens(&mut rewritten);
    concatenated_strings(&rewritten)
        .map(|sql| {
            super::super::parse_with_sources(
                &locations::align_chr_sql(&sql, original),
                true,
                allow_concurrent_detach,
            )
        })
        .unwrap_or_default()
}

pub(in super::super) fn concatenated_strings(tokens: &[Token]) -> Option<String> {
    let mut sql = String::new();
    let mut expect_string = true;
    let mut saw_string = false;
    for token in tokens {
        if matches!(token, Token::Whitespace(_)) {
            continue;
        }
        match token {
            Token::SingleQuotedString(value) if expect_string => {
                sql.push_str(value);
                expect_string = false;
                saw_string = true;
            }
            Token::DollarQuotedString(value) if expect_string => {
                sql.push_str(&value.value);
                expect_string = false;
                saw_string = true;
            }
            Token::StringConcat if !expect_string => expect_string = true,
            _ => return None,
        }
    }
    (saw_string && !expect_string && !sql.is_empty()).then_some(sql)
}
