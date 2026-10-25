use super::Cursor;
use super::reserved::reserved;
use sqlparser::tokenizer::Token;

impl Cursor<'_> {
    pub(in super::super) fn identifier(&mut self) -> bool {
        let Some(Token::Word(word)) = self.tokens.get(self.at) else {
            return false;
        };
        if word.value.is_empty()
            || !(word.quote_style == Some('"')
                || (word.quote_style.is_none() && !reserved(&word.value)))
        {
            return false;
        }
        self.at += 1;
        true
    }
}
