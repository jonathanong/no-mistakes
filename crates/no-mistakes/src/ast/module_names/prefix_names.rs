//! A fatal-only proof for namespace prefixes erased from Oxc's module record.
//! Stop at the first unsupported construct; never search arbitrary source text.
use cursor::{Cursor, Token};
use oxc_span::Span;

mod cursor;
mod specifiers;

pub(super) fn first_legacy_name(source: &str, limit: usize) -> Option<Span> {
    sequence(&mut Cursor::new(source, limit), false)
        .ok()
        .flatten()
}

fn sequence(cursor: &mut Cursor<'_>, nested: bool) -> Result<Option<Span>, ()> {
    loop {
        let token = cursor.statement().ok_or(())?;
        let bad = match token {
            Token::End if !nested => return Ok(None),
            Token::Punctuation('}') if nested => return Ok(None),
            Token::Punctuation(';') => continue,
            Token::Word("namespace" | "module") => namespace(cursor)?,
            Token::Word("declare") => {
                if !matches!(cursor.next(), Some(Token::Word("namespace" | "module"))) {
                    return Err(());
                }
                namespace(cursor)?
            }
            Token::Word("export") => match cursor.next() {
                Some(Token::Word("namespace" | "module")) => namespace(cursor)?,
                Some(Token::Word("declare")) => {
                    if !matches!(cursor.next(), Some(Token::Word("namespace" | "module"))) {
                        return Err(());
                    }
                    namespace(cursor)?
                }
                Some(Token::Punctuation('{')) => specifiers::clause(cursor, false)?,
                Some(Token::Punctuation('*')) => specifiers::all(cursor)?,
                _ => return Err(()),
            },
            Token::Word("import") => {
                if cursor.next() != Some(Token::Punctuation('{')) {
                    return Err(());
                }
                specifiers::clause(cursor, true)?
            }
            _ => return Err(()),
        };
        if bad.is_some() {
            return Ok(bad);
        }
    }
}

fn namespace(cursor: &mut Cursor<'_>) -> Result<Option<Span>, ()> {
    loop {
        match cursor.next() {
            Some(Token::Word(word)) if binding_word(word) => {}
            _ => return Err(()),
        }
        match cursor.next() {
            Some(Token::Punctuation('.')) => continue,
            Some(Token::Punctuation('{')) => return sequence(cursor, true),
            _ => return Err(()),
        }
    }
}

fn binding_word(word: &str) -> bool {
    !matches!(
        word,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}
