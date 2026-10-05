use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};

/// PostgreSQL accepts `FOR <strength> OF a, b [NOWAIT | SKIP LOCKED]`, but
/// sqlparser models `OF` as a single relation. Rewrite the list into one
/// identical locking clause per relation, the form sqlparser already parses.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    let mut result = Vec::with_capacity(tokens.len());
    let mut at = 0;
    let mut changed = false;
    while at < tokens.len() {
        if let Some((end, clauses)) = split_clause(tokens, at) {
            for (index, clause) in clauses.iter().enumerate() {
                if index > 0 {
                    result.push(space_like(&tokens[at]));
                }
                result.extend(clause.iter().cloned());
            }
            at = end;
            changed = true;
        } else {
            result.push(tokens[at].clone());
            at += 1;
        }
    }
    if changed {
        *tokens = result;
    }
}

fn space_like(token: &TokenWithSpan) -> TokenWithSpan {
    TokenWithSpan {
        token: Token::Whitespace(Whitespace::Space),
        span: token.span,
    }
}

const STRENGTHS: &[&[&str]] = &[
    &["update"],
    &["no", "key", "update"],
    &["share"],
    &["key", "share"],
];
const WAIT_POLICIES: &[&[&str]] = &[&["nowait"], &["skip", "locked"]];

/// Return the end of a multi-target locking clause starting at `start` and the
/// per-relation clauses that replace it.
fn split_clause(
    tokens: &[TokenWithSpan],
    start: usize,
) -> Option<(usize, Vec<Vec<TokenWithSpan>>)> {
    if !is_word(&tokens[start].token, "for") {
        return None;
    }
    let mut cursor = Cursor {
        tokens,
        at: start + 1,
    };
    let mut prefix = vec![tokens[start].clone()];
    prefix.extend(cursor.words(STRENGTHS)?);
    prefix.extend(cursor.words(&[&["of"]])?);
    let mut targets = vec![cursor.name()?];
    while cursor.eat(&Token::Comma).is_some() {
        targets.push(cursor.name()?);
    }
    if targets.len() < 2 {
        return None;
    }
    let suffix = cursor.words(WAIT_POLICIES).unwrap_or_default();
    let space = space_like(&tokens[start]);
    let clauses = targets
        .into_iter()
        .map(|target| {
            let mut clause = Vec::new();
            for word in &prefix {
                clause.extend([word.clone(), space.clone()]);
            }
            clause.extend(target);
            for word in &suffix {
                clause.extend([space.clone(), word.clone()]);
            }
            clause
        })
        .collect();
    Some((cursor.at, clauses))
}

struct Cursor<'a> {
    tokens: &'a [TokenWithSpan],
    at: usize,
}

impl Cursor<'_> {
    fn next_significant(&self) -> usize {
        let mut probe = self.at;
        while matches!(self.tokens.get(probe), Some(t) if matches!(t.token, Token::Whitespace(_))) {
            probe += 1;
        }
        probe
    }

    fn eat(&mut self, expected: &Token) -> Option<TokenWithSpan> {
        let probe = self.next_significant();
        let token = self.tokens.get(probe).filter(|t| &t.token == expected)?;
        self.at = probe + 1;
        Some(token.clone())
    }

    /// Consume one of the keyword sequences, returning its words.
    fn words(&mut self, options: &[&[&str]]) -> Option<Vec<TokenWithSpan>> {
        options.iter().find_map(|option| {
            let saved = self.at;
            let mut words = Vec::new();
            for keyword in *option {
                let probe = self.next_significant();
                match self.tokens.get(probe) {
                    Some(t) if is_word(&t.token, keyword) => {
                        words.push(t.clone());
                        self.at = probe + 1;
                    }
                    _ => {
                        self.at = saved;
                        return None;
                    }
                }
            }
            Some(words)
        })
    }

    /// Consume a possibly schema-qualified relation name.
    fn name(&mut self) -> Option<Vec<TokenWithSpan>> {
        let mut name = Vec::new();
        loop {
            let probe = self.next_significant();
            let part = self
                .tokens
                .get(probe)
                .filter(|t| matches!(t.token, Token::Word(_)))?;
            name.push(part.clone());
            self.at = probe + 1;
            match self.eat(&Token::Period) {
                Some(period) => name.push(period),
                None => return Some(name),
            }
        }
    }
}

fn is_word(token: &Token, keyword: &str) -> bool {
    matches!(token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(keyword))
}

#[cfg(test)]
mod tests;
