//! Complete PostgreSQL CREATE/DROP object forms missing from sqlparser.
use super::Cursor;
use sqlparser::tokenizer::Token;
#[path = "regress_create/native.rs"]
mod native;
#[path = "regress_create/table.rs"]
mod table;
pub(super) use native::valid as valid_native;

pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if c.word("DROP") {
        if c.word("DOMAIN") {
            return drop_domain(c);
        }
        if c.word("DATABASE") {
            return drop_database(c);
        }
        return false;
    }
    if !c.word("CREATE") {
        return false;
    }
    let temporary = c.word("TEMP") || c.word("TEMPORARY");
    if temporary {
        return table::parse(c);
    }
    if table::parse(c) {
        return true;
    }
    if c.word("DOMAIN") {
        return c.name() && c.data_type();
    }
    if c.word("TYPE") {
        return base_type(c);
    }
    if c.words(&["MATERIALIZED", "VIEW"]) {
        return materialized_view(c);
    }
    if c.word("ROLE") || c.word("USER") {
        return role(c);
    }
    if c.word("SCHEMA") {
        return schema(c);
    }
    false
}
fn drop_domain(c: &mut Cursor<'_>) -> bool {
    c.words(&["IF", "EXISTS"]);
    if !c.name() {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.name() {
            return false;
        }
    }
    c.drop_behavior();
    true
}
fn drop_database(c: &mut Cursor<'_>) -> bool {
    c.words(&["IF", "EXISTS"]);
    c.identifier() && (!c.token(Token::LParen) || (c.word("FORCE") && c.token(Token::RParen)))
}
fn base_type(c: &mut Cursor<'_>) -> bool {
    if !c.name() || !c.token(Token::LParen) || !type_option(c) {
        return false;
    }
    while c.token(Token::Comma) {
        if !type_option(c) {
            return false;
        }
    }
    c.token(Token::RParen)
}
fn type_option(c: &mut Cursor<'_>) -> bool {
    let key = if c.word("INPUT") || c.word("OUTPUT") {
        1
    } else if c.word("ALIGNMENT") || c.word("STORAGE") {
        2
    } else {
        return false;
    };
    c.token(Token::Eq)
        && if key == 1 {
            c.function_name()
        } else {
            c.word("CHAR")
                || c.word("INT2")
                || c.word("INT4")
                || c.word("DOUBLE")
                || c.word("PLAIN")
                || c.word("EXTERNAL")
                || c.word("EXTENDED")
                || c.word("MAIN")
                || c.word("INTEGER")
        }
}
fn materialized_view(c: &mut Cursor<'_>) -> bool {
    c.words(&["IF", "NOT", "EXISTS"]);
    if !c.name() {
        return false;
    }
    if c.word("USING") && !c.identifier() {
        return false;
    }
    c.word("AS") && c.0.parse_query().is_ok()
}
fn role(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    if c.word("SYSID") {
        return c.integer();
    }
    c.word("NOLOGIN") || c.word("LOGIN")
}

fn schema(c: &mut Cursor<'_>) -> bool {
    c.word("AUTHORIZATION")
        && (c.word("CURRENT_ROLE") || c.word("CURRENT_USER") || c.identifier())
        && c.words(&["CREATE", "SEQUENCE"])
        && c.name()
}
