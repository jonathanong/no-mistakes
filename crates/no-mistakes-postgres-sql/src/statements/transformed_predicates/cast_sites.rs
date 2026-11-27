use sqlparser::ast::CastKind;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Location, Span, Token, TokenWithSpan};

pub(super) fn site(tokens: &[TokenWithSpan], span: Span, kind: &CastKind) -> Option<Location> {
    let cursor = Cursor(tokens);
    let start = tokens.partition_point(|token| token.span.start < span.start);
    if tokens.get(start)?.span.start != span.start {
        return None;
    }
    let mut after = cursor.name(start)?;
    let mut before = start;
    let mut wrappers = Vec::new();
    while let Some(open) = cursor.previous(before) {
        if cursor.token(open) != Some(&Token::LParen) {
            break;
        }
        let prefix = cursor
            .previous(open)
            .and_then(|index| cast_kind(cursor.token(index)?).map(|kind| (index, kind)));
        if prefix.is_none() && !cursor.grouping(open) {
            break;
        }
        before = prefix.as_ref().map_or(open, |(index, _)| *index);
        wrappers.push(prefix);
    }
    let mut closed = 0;
    let mut prefix = None;
    let mut postfix = None;
    loop {
        match cursor.token(after) {
            Some(Token::DoubleColon) => {
                let marker = after;
                after = cursor.data_type(cursor.next(after + 1))?;
                postfix = Some((marker, after));
            }
            Some(Token::Word(word)) if word.keyword == Keyword::AS => {
                let Some(Some((marker, syntax))) = wrappers.get(closed) else {
                    break;
                };
                let end = cursor.data_type(cursor.next(after + 1))?;
                if cursor.token(end) != Some(&Token::RParen) {
                    break;
                }
                after = cursor.next(end + 1);
                closed += 1;
                if syntax == kind {
                    prefix = Some((*marker, after));
                }
            }
            Some(Token::RParen) if wrappers.get(closed) == Some(&None) => {
                after = cursor.next(after + 1);
                closed += 1;
            }
            _ => break,
        }
    }
    let (marker, end) = if *kind == CastKind::DoubleColon {
        postfix
    } else {
        prefix
    }?;
    let last = cursor.previous(end)?;
    (tokens[last].span.end >= span.end).then_some(tokens[marker].span.start)
}

struct Cursor<'a>(&'a [TokenWithSpan]);

impl Cursor<'_> {
    fn token(&self, index: usize) -> Option<&Token> {
        self.0.get(index).map(|token| &token.token)
    }
    fn next(&self, mut index: usize) -> usize {
        while matches!(self.token(index), Some(Token::Whitespace(_))) {
            index += 1;
        }
        index
    }
    fn previous(&self, index: usize) -> Option<usize> {
        let mut index = index.checked_sub(1)?;
        while matches!(self.token(index), Some(Token::Whitespace(_))) {
            index = index.checked_sub(1)?;
        }
        Some(index)
    }
    fn name(&self, start: usize) -> Option<usize> {
        let end = self.arrays(self.name_end(start)?)?;
        (self.token(end) != Some(&Token::LParen)).then_some(end)
    }
    fn name_end(&self, start: usize) -> Option<usize> {
        if !matches!(self.token(start), Some(Token::Word(word)) if word.keyword != Keyword::AS) {
            return None;
        }
        let mut end = self.next(start + 1);
        while self.token(end) == Some(&Token::Period) {
            let part = self.next(end + 1);
            if !matches!(self.token(part), Some(Token::Word(_))) {
                return None;
            }
            end = self.next(part + 1);
        }
        Some(end)
    }
    fn data_type(&self, start: usize) -> Option<usize> {
        let mut end = self.name_end(start)?;
        if matches!(self.token(start), Some(Token::Word(word)) if matches!(word.keyword, Keyword::CHARACTER | Keyword::CHAR | Keyword::BIT))
            && matches!(self.token(end), Some(Token::Word(word)) if word.keyword == Keyword::VARYING)
        {
            end = self.next(end + 1);
        }
        if self.token(end) == Some(&Token::LParen) {
            end = self.next(end + 1);
            loop {
                if !matches!(self.token(end), Some(Token::Number(_, _))) {
                    return None;
                }
                end = self.next(end + 1);
                match self.token(end) {
                    Some(Token::Comma) => end = self.next(end + 1),
                    Some(Token::RParen) => {
                        end = self.next(end + 1);
                        break;
                    }
                    _ => return None,
                }
            }
        }
        self.arrays(end)
    }
    fn arrays(&self, mut end: usize) -> Option<usize> {
        while self.token(end) == Some(&Token::LBracket) {
            let close = self.next(end + 1);
            if self.token(close) != Some(&Token::RBracket) {
                return None;
            }
            end = self.next(close + 1);
        }
        Some(end)
    }
    fn grouping(&self, open: usize) -> bool {
        self.previous(open)
            .is_none_or(|index| match self.token(index) {
                Some(Token::Word(word)) => {
                    word.quote_style.is_none()
                        && matches!(
                            word.keyword,
                            Keyword::WHERE
                                | Keyword::ON
                                | Keyword::AND
                                | Keyword::OR
                                | Keyword::NOT
                                | Keyword::HAVING
                                | Keyword::WHEN
                                | Keyword::THEN
                                | Keyword::ELSE
                                | Keyword::SELECT
                                | Keyword::AS
                        )
                }
                _ => true,
            })
    }
}

fn cast_kind(token: &Token) -> Option<CastKind> {
    let Token::Word(word) = token else {
        return None;
    };
    if word.quote_style.is_some() {
        return None;
    }
    match word.keyword {
        Keyword::CAST => Some(CastKind::Cast),
        Keyword::TRY_CAST => Some(CastKind::TryCast),
        Keyword::SAFE_CAST => Some(CastKind::SafeCast),
        _ => None,
    }
}
