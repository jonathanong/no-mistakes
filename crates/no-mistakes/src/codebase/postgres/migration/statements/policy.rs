//! Narrow syntactic projections cover PostgreSQL utility grammar gaps without
//! pretending unsupported statements have an unrelated AST shape.
use crate::codebase::postgres::parse::{
    normalize_copy_data_for_tokens, unicode::tokenize_raw_unicode,
};
use crate::codebase::postgres::types::{
    SqlPolicyClause, SqlPolicyClauseUse, SqlSchemaFileFacts, SqlStatementKind,
};
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::borrow::Cow;

mod kinds;
mod settings;
use kinds::{new_kind, word};

pub(crate) const SUPPORTED_KINDS: &[&str] = &[
    "CREATE TABLE",
    "ALTER TABLE",
    "CREATE INDEX",
    "CREATE VIEW",
    "TRUNCATE",
    "DROP INDEX",
    "DROP VIEW",
    "CREATE DATABASE",
    "DROP DATABASE",
    "ALTER DATABASE",
    "ALTER SYSTEM",
    "CREATE SCHEMA",
    "ALTER SCHEMA",
    "DROP SCHEMA",
    "CREATE TRIGGER",
    "DROP TRIGGER",
    "CREATE FUNCTION",
    "CREATE PROCEDURE",
    "DROP FUNCTION",
    "DROP PROCEDURE",
    "DROP TABLE",
    "CREATE TYPE",
    "DROP TYPE",
    "ALTER VIEW",
    "ALTER TRIGGER",
    "ALTER FUNCTION",
    "ALTER PROCEDURE",
    "ALTER INDEX",
    "ALTER TYPE",
];

pub(in super::super) fn record(sql: &str, facts: &mut SqlSchemaFileFacts) {
    direct(sql, false, facts);
    // Dollar-quoted DO bodies are already peeled into AST metadata. Only the
    // new token facts need this projection; do not duplicate existing metadata.
    for body in super::super::dynamic::peeled_schema_bodies(sql) {
        let mut nested = SqlSchemaFileFacts::default();
        direct(&body.sql, true, &mut nested);
        for mut kind in nested.statement_kinds {
            kind.line = body.source_line(kind.line);
            facts.statement_kinds.push(kind);
        }
        for mut used in nested.clause_uses {
            used.line = body.source_line(used.line);
            used.column = 1;
            facts.clause_uses.push(used);
        }
        for mut setting in nested.setting_uses {
            setting.line = body.source_line(setting.line);
            facts.setting_uses.push(setting);
        }
    }
}

fn direct(sql: &str, procedural: bool, facts: &mut SqlSchemaFileFacts) {
    let normalized = normalize_copy_data_for_tokens(sql);
    for code in significant_chunks(&tokenize_raw_unicode(sql)) {
        let start = statement_start(&code, procedural);
        let statement = start.map(|start| &code[start..]).unwrap_or_default();
        if let Some(kind) = new_kind(statement) {
            facts.statement_kinds.push(SqlStatementKind {
                kind: kind.into(),
                line: statement[0].span.start.line.max(1) as usize,
            });
        }
        if matches!(normalized, Cow::Borrowed(_)) {
            facts.clause_uses.extend(extension_version(statement));
        }
        settings::collect(statement, &code, &mut facts.setting_uses);
    }
    // Keep the historical statement and setting projections when COPY has data.
    if let Cow::Owned(normalized) = normalized {
        let tokens = tokenize_raw_unicode(&normalized);
        facts
            .clause_uses
            .extend(significant_chunks(&tokens).filter_map(|code| {
                let start = statement_start(&code, procedural)?;
                extension_version(&code[start..])
            }));
    }
}

fn significant_chunks(tokens: &[TokenWithSpan]) -> impl Iterator<Item = Vec<&TokenWithSpan>> {
    tokens
        .split(|token| matches!(token.token, Token::SemiColon))
        .map(|chunk| {
            chunk
                .iter()
                .filter(|token| !matches!(token.token, Token::Whitespace(_)))
                .collect()
        })
}

fn extension_version(statement: &[&TokenWithSpan]) -> Option<SqlPolicyClauseUse> {
    let [first, second, ..] = statement else {
        return None;
    };
    if !word(first, "CREATE") || !word(second, "EXTENSION") {
        return None;
    }
    let tokens: Vec<_> = statement.iter().map(|token| &token.token).collect();
    crate::codebase::postgres::parse::extension_has_version(&tokens).then_some(SqlPolicyClauseUse {
        clause: SqlPolicyClause::CreateExtensionVersion,
        line: first.span.start.line.max(1) as usize,
        column: first.span.start.column.max(1) as usize,
    })
}

fn statement_start(code: &[&TokenWithSpan], procedural: bool) -> Option<usize> {
    let first = *code.first()?;
    if !procedural
        && !["BEGIN", "IF", "ELSIF", "ELSE"]
            .iter()
            .any(|name| word(first, name))
    {
        return Some(0);
    }
    let mut at = 0;
    while at < code.len() {
        if [
            "CREATE", "ALTER", "DROP", "SET", "SELECT", "PERFORM", "RETURN",
        ]
        .iter()
        .any(|name| word(code[at], name))
        {
            // Once a SQL header starts, THEN/ELSE belong to its expressions.
            return Some(at);
        }
        if word(code[at], "IF") || word(code[at], "ELSIF") {
            at += code[at..].iter().position(|token| word(token, "THEN"))?;
        }
        at += 1;
    }
    None
}
