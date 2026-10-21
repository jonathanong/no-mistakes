use super::{
    executed_query_text, executor_bindings, extract_embedded_sql_from_source, is_database_call,
    sql_text, EmbeddedSqlOptions,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

pub(super) fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("fixture")
}

fn extract(name: &str) -> super::EmbeddedSqlFileFacts {
    extract_from_path(&fixture(name))
}

pub(super) fn extract_from_path(path: &Path) -> super::EmbeddedSqlFileFacts {
    extract_embedded_sql_from_source(
        path,
        &std::fs::read_to_string(path).expect("fixture"),
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
}

fn parse_expr(source: &str) -> (Allocator, String) {
    (Allocator::default(), format!("const value = {source};"))
}

fn first_init<'a>(program: &'a oxc_ast::ast::Program<'a>) -> &'a Expression<'a> {
    match &program.body[0] {
        oxc_ast::ast::Statement::VariableDeclaration(declaration) => {
            declaration.declarations[0].init.as_ref().expect("init")
        }
        _ => panic!("expected variable"),
    }
}

mod builder_text;
mod iife_fail_closed;
mod part1;
mod part2;
mod part3;
mod provenance_review;
