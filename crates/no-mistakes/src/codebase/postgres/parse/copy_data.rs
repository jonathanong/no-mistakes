use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::Token;

pub(super) fn is_copy_stdin(sql: &str) -> bool {
    let tokens = super::unicode::tokenize_raw_unicode(sql);
    let mut tokens = tokens
        .iter()
        .filter(|t| !matches!(t.token, Token::Whitespace(_)));
    if !matches!(tokens.next().map(|t| &t.token), Some(Token::Word(word)) if word.keyword == Keyword::COPY)
    {
        return false;
    }
    let mut depth = 0usize;
    let mut from = false;
    for token in tokens {
        match &token.token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            Token::Word(word) if depth == 0 => {
                if from && word.keyword == Keyword::STDIN {
                    return true;
                }
                from = word.keyword == Keyword::FROM;
            }
            _ => from = false,
        }
    }
    false
}

pub(super) fn payload_end(sql: &str, start: usize) -> (usize, usize) {
    let mut at = start;
    for line in sql[start..].split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "\\." {
            return (at, at + line.len());
        }
        at += line.len();
    }
    (sql.len(), sql.len())
}
