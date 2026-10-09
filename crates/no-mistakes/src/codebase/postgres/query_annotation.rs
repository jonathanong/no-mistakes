//! Annotation-only syntactic summaries. These never claim complete SQL for
//! structural PostgreSQL checks, and are collected from the shared OXC program.
mod evaluate;
mod expressions;
pub(crate) mod project;
mod statements;

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct QueryAnnotationFileFacts {
    pub(super) globals: BTreeMap<String, Expr>,
    pub(super) roots: Vec<Step>,
    pub(super) trusted_tags: BTreeSet<String>,
    pub calls: BTreeMap<(u32, String), Option<String>>,
}

#[derive(Clone, Debug)]
pub(super) enum Expr {
    Unknown,
    Unsupported,
    Text(String),
    Name(String),
    Template(Vec<Expr>),
    Tagged(String, String),
    Append(Box<Expr>, Box<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        line: u32,
        spelling: String,
    },
    Function(Function),
    Children(Vec<Expr>),
}

#[derive(Clone, Debug)]
pub(super) struct Function {
    pub params: Vec<String>,
    pub body: Vec<Step>,
    pub supported: bool,
    pub asynchronous: bool,
    pub self_name: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) enum Step {
    Bind(String, Expr),
    Append(String, Expr),
    Effect(Expr),
    Return(Expr),
    Unsupported,
}

pub(crate) fn collect(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    options: &super::EmbeddedSqlOptions,
) -> QueryAnnotationFileFacts {
    statements::collect(program, source, options)
}
