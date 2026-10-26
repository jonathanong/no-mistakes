//! PostgreSQL 17 ALTER forms observed in the regression SQL corpus.
use super::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if !c.word("ALTER") {
        return false;
    }
    if c.word("TABLE") {
        return table(c);
    }
    if c.word("TYPE") {
        return ty(c);
    }
    if c.word("INDEX") {
        return index(c);
    }
    false
}

fn table(c: &mut Cursor<'_>) -> bool {
    if c.words(&["ALL", "IN", "TABLESPACE"]) {
        return c.identifier() && c.words(&["SET", "TABLESPACE"]) && c.identifier();
    }
    c.words(&["IF", "EXISTS"]);
    c.words(&["ONLY"]);
    if !c.name() {
        return false;
    }
    c.token(Token::Mul); // inherited descendant marker: table_name *
    if c.word("ALTER") {
        return table_alter(c);
    }
    if c.word("ATTACH") {
        return c.word("PARTITION")
            && c.name()
            && c.words(&["FOR", "VALUES", "WITH"])
            && partition_bound(c);
    }
    if c.words(&["DETACH", "PARTITION"]) {
        return c.name() && {
            c.word("CONCURRENTLY") || c.word("FINALIZE") || c.0.peek_token().token == Token::EOF
        };
    }
    if c.words(&["CLUSTER", "ON"]) {
        return c.name();
    }
    if c.word("INHERIT") {
        return c.name();
    }
    if c.words(&["NOT", "OF"]) {
        return true;
    }
    if c.word("OF") {
        return c.name();
    }
    if c.word("RESET") {
        return option_names(c);
    }
    if c.word("SET") {
        if c.words(&["WITHOUT", "CLUSTER"]) {
            return true;
        }
        if c.words(&["ACCESS", "METHOD"]) || c.word("SCHEMA") || c.word("TABLESPACE") {
            return c.name();
        }
        return options(c);
    }
    false
}

fn table_alter(c: &mut Cursor<'_>) -> bool {
    if c.word("CONSTRAINT") {
        return c.identifier() && c.word("DEFERRABLE") && c.words(&["INITIALLY", "DEFERRED"]);
    }
    c.word("COLUMN");
    if !c.identifier() {
        return false;
    }
    if c.words(&["SET", "STATISTICS"]) {
        return signed_integer(c);
    }
    c.word("SET") && options(c)
}

fn ty(c: &mut Cursor<'_>) -> bool {
    if !c.name() {
        return false;
    }
    if c.word("ADD") {
        return c.word("ATTRIBUTE") && c.identifier() && c.data_type() && {
            c.drop_behavior();
            true
        };
    }
    if c.word("ALTER") {
        return c.word("ATTRIBUTE") && c.identifier() && c.word("TYPE") && c.data_type() && {
            c.drop_behavior();
            true
        };
    }
    if c.word("DROP") {
        return c.word("ATTRIBUTE")
            && {
                c.words(&["IF", "EXISTS"]);
                c.identifier()
            }
            && {
                c.drop_behavior();
                true
            };
    }
    if c.words(&["OWNER", "TO"]) {
        return c.identifier();
    }
    if c.words(&["RENAME", "ATTRIBUTE"]) {
        return c.identifier() && c.word("TO") && c.identifier();
    }
    c.word("SET") && options(c)
}

fn index(c: &mut Cursor<'_>) -> bool {
    if c.words(&["ALL", "IN", "TABLESPACE"]) {
        return c.identifier() && c.words(&["SET", "TABLESPACE"]) && c.identifier();
    }
    c.words(&["IF", "EXISTS"]);
    if !c.name() {
        return false;
    }
    if c.word("ALTER") {
        return c.word("COLUMN")
            && c.integer()
            && c.words(&["SET", "STATISTICS"])
            && signed_integer(c);
    }
    if c.words(&["SET", "TABLESPACE"]) {
        return c.identifier();
    }
    c.word("SET") && options(c)
}

fn signed_integer(c: &mut Cursor<'_>) -> bool {
    if !c.token(Token::Minus) {
        c.token(Token::Plus);
    }
    c.integer()
}
fn option_names(c: &mut Cursor<'_>) -> bool {
    if !c.token(Token::LParen) || !c.name() {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.name() {
            return false;
        }
    }
    c.token(Token::RParen)
}
fn options(c: &mut Cursor<'_>) -> bool {
    if !c.token(Token::LParen) || !option(c) {
        return false;
    }
    while c.token(Token::Comma) {
        if !option(c) {
            return false;
        }
    }
    c.token(Token::RParen)
}
fn option(c: &mut Cursor<'_>) -> bool {
    c.name()
        && c.token(Token::Eq)
        && (signed_integer(c)
            || c.string()
            || c.identifier()
            || c.word("ON")
            || c.word("OFF")
            || c.word("TRUE")
            || c.word("FALSE"))
}
fn partition_bound(c: &mut Cursor<'_>) -> bool {
    c.token(Token::LParen)
        && c.word("MODULUS")
        && c.integer()
        && c.token(Token::Comma)
        && c.word("REMAINDER")
        && c.integer()
        && c.token(Token::RParen)
}
#[cfg(test)]
#[path = "regress_alter_tests.rs"]
mod tests;
