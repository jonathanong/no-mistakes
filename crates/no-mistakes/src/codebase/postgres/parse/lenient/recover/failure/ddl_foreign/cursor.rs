use sqlparser::tokenizer::Token;

pub(super) struct Cursor<'a> {
    pub(super) tokens: &'a [&'a Token],
    pub(super) at: usize,
}

impl Cursor<'_> {
    pub(super) fn option_value(&mut self, required: bool) -> bool {
        self.identifier() && (!required || self.string())
    }

    pub(super) fn options(&mut self, alter: bool) -> bool {
        if !self.left_paren() {
            return false;
        }
        if alter && self.right_paren() {
            return false;
        }
        loop {
            if alter && (self.word("ADD") || self.word("SET")) {
                if !self.option_value(true) {
                    return false;
                }
            } else if alter && self.word("DROP") {
                if !self.option_value(false) {
                    return false;
                }
            } else if !self.option_value(true) {
                return false;
            }
            if self.right_paren() {
                return true;
            }
            if !self.comma() {
                return false;
            }
        }
    }
    pub(super) fn names_in_parens(&mut self) -> bool {
        if !self.left_paren() || !self.identifier() {
            return false;
        }
        while self.comma() {
            if !self.identifier() {
                return false;
            }
        }
        self.right_paren()
    }
    pub(super) fn roles(&mut self) -> bool {
        if !self.owned_role() {
            return false;
        }
        while self.comma() {
            if !self.owned_role() {
                return false;
            }
        }
        true
    }
    pub(super) fn names(&mut self) -> bool {
        if !self.name() {
            return false;
        }
        while self.comma() {
            if !self.name() {
                return false;
            }
        }
        true
    }
    pub(super) fn role(&mut self) -> bool {
        ["CURRENT_ROLE", "CURRENT_USER", "SESSION_USER"]
            .iter()
            .any(|word| self.word(word))
            || self.identifier()
    }
    pub(super) fn mapping_role(&mut self) -> bool {
        [
            "CURRENT_ROLE",
            "CURRENT_USER",
            "SESSION_USER",
            "USER",
            "PUBLIC",
        ]
        .iter()
        .any(|word| self.word(word))
            || self.identifier()
    }
    fn owned_role(&mut self) -> bool {
        ["CURRENT_ROLE", "CURRENT_USER", "SESSION_USER"]
            .iter()
            .any(|word| self.word(word))
            || self.identifier()
    }
    pub(super) fn name(&mut self) -> bool {
        self.identifier() && (!self.period() || self.identifier())
    }
    pub(super) fn words(&mut self, words: &[&str]) -> bool {
        let start = self.at;
        if words.iter().all(|word| self.word(word)) {
            true
        } else {
            self.at = start;
            false
        }
    }
    pub(super) fn word(&mut self, value: &str) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Word(w)) if w.quote_style.is_none() && w.value.eq_ignore_ascii_case(value))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn identifier(&mut self) -> bool {
        let Some(Token::Word(word)) = self.tokens.get(self.at) else {
            return false;
        };
        let keyword_identifier = word.keyword == sqlparser::keywords::Keyword::NoKeyword
            || [
                "int",
                "integer",
                "text",
                "boolean",
                "varchar",
                "char",
                "date",
                "timestamp",
                "interval",
                "numeric",
                "decimal",
                "real",
                "smallint",
                "bigint",
                "double",
                "precision",
                "local",
                "user",
                "public",
            ]
            .iter()
            .any(|value| word.value.eq_ignore_ascii_case(value));
        if word.value.is_empty()
            || !(word.quote_style == Some('"')
                || (word.quote_style.is_none() && keyword_identifier))
        {
            return false;
        }
        self.at += 1;
        true
    }
    pub(super) fn number(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Number(n, _)) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn string(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::SingleQuotedString(_))) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn punctuation(&mut self, expected: Token) -> bool {
        if self.tokens.get(self.at).is_some_and(|t| **t == expected) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn comma(&mut self) -> bool {
        self.punctuation(Token::Comma)
    }
    pub(super) fn period(&mut self) -> bool {
        self.punctuation(Token::Period)
    }
    pub(super) fn left_paren(&mut self) -> bool {
        self.punctuation(Token::LParen)
    }
    pub(super) fn right_paren(&mut self) -> bool {
        self.punctuation(Token::RParen)
    }
    pub(super) fn left_bracket(&mut self) -> bool {
        self.punctuation(Token::LBracket)
    }
    pub(super) fn right_bracket(&mut self) -> bool {
        self.punctuation(Token::RBracket)
    }
}
