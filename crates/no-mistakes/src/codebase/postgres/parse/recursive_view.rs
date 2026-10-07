use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};

/// sqlparser 0.63 has no recursive-view modifier. Ignore only that modifier in
/// CREATE [OR REPLACE] RECURSIVE VIEW headers so the existing CreateView parser
/// and projection can run. Keep all spans and the original source SQL intact;
/// WITH RECURSIVE queries and quoted/string occurrences are untouched.
pub(super) fn normalize(tokens: &mut [TokenWithSpan]) {
    let mut preceding = [Keyword::NoKeyword; 4];
    let mut previous_index = 0;
    for index in 0..tokens.len() {
        let keyword = match &tokens[index].token {
            Token::Whitespace(_) => continue,
            Token::Word(word) if word.quote_style.is_none() => word.keyword,
            _ => Keyword::NoKeyword,
        };
        if keyword == Keyword::VIEW
            && (matches!(preceding, [_, _, Keyword::CREATE, Keyword::RECURSIVE])
                || preceding
                    == [
                        Keyword::CREATE,
                        Keyword::OR,
                        Keyword::REPLACE,
                        Keyword::RECURSIVE,
                    ])
        {
            tokens[previous_index].token = Token::Whitespace(Whitespace::Space);
        }
        preceding.rotate_left(1);
        preceding[3] = keyword;
        previous_index = index;
    }
}
