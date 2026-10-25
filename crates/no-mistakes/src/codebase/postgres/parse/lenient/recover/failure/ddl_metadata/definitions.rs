use super::Cursor;
use sqlparser::tokenizer::Token;
pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if c.words(&["DROP", "TRANSFORM"]) {
        c.words(&["IF", "EXISTS"]);
        if !c.word("FOR") || !c.data_type() || !c.word("LANGUAGE") || !c.identifier() {
            return false;
        }
        c.drop_behavior();
        return true;
    }
    if !c.word("CREATE") {
        return false;
    }
    if c.words(&["ACCESS", "METHOD"]) {
        return c.identifier()
            && c.word("TYPE")
            && (c.word("TABLE") || c.word("INDEX"))
            && c.word("HANDLER")
            && c.name();
    }
    let replace = c.words(&["OR", "REPLACE"]);
    if c.word("TRANSFORM") {
        return transform(c);
    }
    if c.word("AGGREGATE") {
        return aggregate(c);
    }
    if replace {
        return false;
    }
    if c.word("CAST") {
        return cast(c);
    }
    c.word("DEFAULT");
    c.word("CONVERSION")
        && c.name()
        && c.word("FOR")
        && c.string()
        && c.word("TO")
        && c.string()
        && c.word("FROM")
        && c.name()
}
fn cast(c: &mut Cursor<'_>) -> bool {
    if !c.token(Token::LParen)
        || !c.data_type()
        || !c.word("AS")
        || !c.data_type()
        || !c.token(Token::RParen)
    {
        return false;
    }
    if !(c.words(&["WITHOUT", "FUNCTION"])
        || c.words(&["WITH", "INOUT"])
        || (c.words(&["WITH", "FUNCTION"]) && c.signature()))
    {
        return false;
    }
    !c.word("AS") || c.word("ASSIGNMENT") || c.word("IMPLICIT")
}
fn transform(c: &mut Cursor<'_>) -> bool {
    if !c.word("FOR")
        || !c.data_type()
        || !c.word("LANGUAGE")
        || !c.identifier()
        || !c.token(Token::LParen)
    {
        return false;
    }
    let from = c.word("FROM");
    if !(from || c.word("TO")) || !c.words(&["SQL", "WITH", "FUNCTION"]) || !c.signature() {
        return false;
    }
    if c.token(Token::Comma)
        && (!(if from { c.word("TO") } else { c.word("FROM") })
            || !c.words(&["SQL", "WITH", "FUNCTION"])
            || !c.signature())
    {
        return false;
    }
    c.token(Token::RParen)
}
fn aggregate(c: &mut Cursor<'_>) -> bool {
    // Validate the basic required-parameter grammar; ordered-set and moving
    // aggregates remain unsupported until their distinct grammars are modeled.
    if !c.name()
        || !c.token(Token::LParen)
        || !(c.token(Token::Mul) || c.types())
        || !c.token(Token::RParen)
        || !c.token(Token::LParen)
    {
        return false;
    }
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let key = if c.word("SFUNC") {
            "SFUNC"
        } else if c.word("STYPE") {
            "STYPE"
        } else if c.word("INITCOND") {
            "INITCOND"
        } else if c.word("FINALFUNC") {
            "FINALFUNC"
        } else {
            return false;
        };
        seen.insert(key);
        if !c.token(Token::Eq) {
            return false;
        }
        let value = match key {
            "STYPE" => c.data_type(),
            "INITCOND" => c.string(),
            _ => c.name(),
        };
        if !value {
            return false;
        }
        if !c.token(Token::Comma) {
            break;
        }
    }
    seen.contains("SFUNC") && seen.contains("STYPE") && c.token(Token::RParen)
}
