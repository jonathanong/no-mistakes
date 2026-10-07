//! Metadata grammar borrows the same prepared parser and source locations.
mod literal;
use super::{
    expressions::{identifier, name},
    locations::Locations,
    type_facts,
    types::*,
};
use sqlparser::{ast::Statement, keywords::Keyword, parser::Parser, tokenizer::Token};

pub(super) fn starts(parser: &Parser<'_>) -> bool {
    let is = |at, expected| matches!(parser.peek_nth_token(at).token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected);
    is(0, Keyword::COMMENT) || (is(0, Keyword::ALTER) && is(1, Keyword::INDEX))
}

pub(super) fn collect(
    parser: &mut Parser<'_>,
    locations: &Locations<'_>,
) -> Result<PostgresSqlStatementKind, String> {
    let result = if parser.parse_keyword(Keyword::COMMENT) {
        comment(parser, locations)
    } else {
        alter_index(parser)
    };
    if result.is_err()
        && parser.token_at(parser.index().saturating_sub(1)).token == Token::SemiColon
    {
        // A missing target/value can consume its delimiter. Keep recovery from
        // swallowing the next independent statement along with that failure.
        parser.prev_token();
    }
    result
}

fn comment(
    parser: &mut Parser<'_>,
    locations: &Locations<'_>,
) -> Result<PostgresSqlStatementKind, String> {
    parser
        .expect_keyword_is(Keyword::ON)
        .map_err(|e| e.to_string())?;
    if matches!(parser.peek_token().token, Token::Word(word) if word.quote_style.is_none() && [Keyword::FUNCTION, Keyword::PROCEDURE].contains(&word.keyword))
    {
        return routine_comment(parser, locations);
    }
    let object = parser
        .parse_one_of_keywords(&[
            Keyword::COLLATION,
            Keyword::COLUMN,
            Keyword::DATABASE,
            Keyword::DOMAIN,
            Keyword::EXTENSION,
            Keyword::INDEX,
            Keyword::MATERIALIZED,
            Keyword::ROLE,
            Keyword::SCHEMA,
            Keyword::SEQUENCE,
            Keyword::TABLE,
            Keyword::TYPE,
            Keyword::VIEW,
        ])
        .ok_or("Expected a supported COMMENT object type")?;
    let object_type = if object == Keyword::MATERIALIZED {
        parser
            .expect_keyword_is(Keyword::VIEW)
            .map_err(|e| e.to_string())?;
        "MATERIALIZED VIEW".into()
    } else {
        object.to_string()
    };
    let target = parser.parse_object_name(false).map_err(|e| e.to_string())?;
    parser
        .expect_keyword_is(Keyword::IS)
        .map_err(|e| e.to_string())?;
    let comment = literal::text(parser, locations)?;
    Ok(PostgresSqlStatementKind::Comment {
        comment: PostgresSqlComment {
            object_type,
            name: name(&target),
            arguments: None,
            comment,
        },
    })
}

fn routine_comment(
    parser: &mut Parser<'_>,
    locations: &Locations<'_>,
) -> Result<PostgresSqlStatementKind, String> {
    // DROP's signature grammar already owns argument modes, names and types.
    // Only borrow that grammar; no DROP facts or execution semantics are emitted.
    let descriptor = parser.parse_drop().map_err(|e| e.to_string())?;
    let (object_type, descriptors, invalid) = match descriptor {
        Statement::DropFunction(value) => (
            "FUNCTION",
            value.func_desc,
            value.if_exists || value.drop_behavior.is_some(),
        ),
        Statement::DropProcedure {
            if_exists,
            proc_desc,
            drop_behavior,
        } => ("PROCEDURE", proc_desc, if_exists || drop_behavior.is_some()),
        _ => return Err("Expected a FUNCTION or PROCEDURE comment target".into()),
    };
    if invalid || descriptors.len() != 1 {
        return Err("COMMENT requires one routine target without DROP modifiers".into());
    }
    let descriptor = &descriptors[0];
    let mut arguments = None;
    if let Some(args) = &descriptor.args {
        let mut projected = Vec::new();
        for arg in args {
            if arg.default_expr.is_some() {
                return Err("COMMENT routine signatures do not allow argument defaults".into());
            }
            projected.push(PostgresSqlFunctionArgument {
                name: arg.name.as_ref().map(identifier),
                mode: arg.mode.as_ref().map(ToString::to_string),
                data_type: type_facts::data_type(&arg.data_type, locations),
                default: None,
            });
        }
        arguments = Some(projected);
    }
    parser
        .expect_keyword_is(Keyword::IS)
        .map_err(|e| e.to_string())?;
    let comment = literal::text(parser, locations)?;
    Ok(PostgresSqlStatementKind::Comment {
        comment: PostgresSqlComment {
            object_type: object_type.into(),
            name: name(&descriptor.name),
            arguments,
            comment,
        },
    })
}

fn alter_index(parser: &mut Parser<'_>) -> Result<PostgresSqlStatementKind, String> {
    parser
        .expect_keywords(&[Keyword::ALTER, Keyword::INDEX])
        .map_err(|e| e.to_string())?;
    let if_exists = parser.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
    let index = parser.parse_object_name(false).map_err(|e| e.to_string())?;
    let operation = if parser.parse_keywords(&[Keyword::ATTACH, Keyword::PARTITION]) {
        if if_exists {
            return Err("ALTER INDEX ATTACH PARTITION does not allow IF EXISTS".into());
        }
        PostgresSqlAlterIndexOperation::AttachPartition {
            partition: name(&parser.parse_object_name(false).map_err(|e| e.to_string())?),
        }
    } else {
        parser
            .expect_keywords(&[Keyword::RENAME, Keyword::TO])
            .map_err(|e| e.to_string())?;
        PostgresSqlAlterIndexOperation::Rename {
            name: name(&sqlparser::ast::ObjectName(vec![
                sqlparser::ast::ObjectNamePart::Identifier(
                    parser.parse_identifier().map_err(|e| e.to_string())?,
                ),
            ])),
        }
    };
    Ok(PostgresSqlStatementKind::AlterIndex {
        index: PostgresSqlAlterIndex {
            name: name(&index),
            if_exists,
            operation,
        },
    })
}
