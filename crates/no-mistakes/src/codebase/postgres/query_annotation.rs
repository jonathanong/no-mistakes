//! Annotation-only syntactic summaries. These never claim complete SQL for
//! structural PostgreSQL checks, and are collected from the shared OXC program.
mod coverage;
mod evaluate;
mod exports;
mod expressions;
pub(crate) mod project;
mod statements;
mod trust;

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct QueryAnnotationFileFacts {
    pub(super) globals: BTreeMap<String, Expr>,
    pub(super) roots: Vec<Step>,
    pub(super) unmodeled_calls: Vec<Expr>,
    pub(super) trusted_tags: BTreeSet<String>,
    pub(super) legacy_tag_spans: BTreeMap<String, u32>,
    pub(super) raw_tag_reassigned: bool,
    pub calls: BTreeMap<u32, Option<String>>,
}

#[derive(Clone, Debug)]
pub(super) enum Expr {
    Unknown,
    Unsupported,
    Text(String),
    Name(String),
    Template(Vec<Expr>),
    Tagged(String, Vec<Expr>, Vec<Expr>),
    Await(Box<Expr>),
    Spread(Box<Expr>),
    Index(Box<Expr>, usize),
    OpaqueCallback(Box<Expr>),
    Append(Box<Expr>, Box<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        start: u32,
    },
    Function(Function),
    Children(Vec<Expr>),
    Alternatives(Vec<Expr>),
    Opaque(Vec<Expr>),
}

#[derive(Clone, Debug)]
pub(super) struct Function {
    pub start: u32,
    pub params: Vec<String>,
    pub body: Vec<Step>,
    pub supported: bool,
    pub asynchronous: bool,
    pub arrow: bool,
    pub self_name: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) enum Step {
    Reserve(Vec<String>),
    Hoisted(String, Expr),
    Var(String),
    Bind(String, Expr),
    Append(String, Expr),
    Effect(Expr),
    Return(Expr),
    Unsupported,
    PotentialCalls(Vec<u32>),
}

pub(crate) fn collect(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    options: &super::EmbeddedSqlOptions,
) -> QueryAnnotationFileFacts {
    statements::collect(program, source, options)
}
