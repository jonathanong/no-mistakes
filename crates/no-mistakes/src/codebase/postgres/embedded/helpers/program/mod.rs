//! A closed, data-only model of a helper function's body.
//!
//! The per-file walk lowers each top-level helper once ([`lower`]). The
//! prepared projection later executes the model against the abstract
//! arguments of one call site ([`exec`]), forking at tests it cannot decide.
//! Anything outside the model lowers to `Opaque`, which fails the execution
//! only when a path actually reaches it, so unrelated unsupported code in a
//! pruned branch does not block recovery.

use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;

pub(crate) mod exec;
pub(crate) mod lower;

/// Physical position of an expression in the helper's own module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mark {
    pub(crate) line: u32,
    pub(crate) origin: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Program {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Vec<Stmt>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Param {
    pub(crate) name: String,
    /// Annotated as the `SQLStatement` type: the only parameter that may
    /// receive a caller's builder.
    pub(crate) builder: bool,
    pub(crate) default: Option<Expr>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SwitchArm {
    pub(crate) cases: Vec<String>,
    pub(crate) value: Expr,
    pub(crate) mark: Mark,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Stmt {
    Let(String, Expr),
    Expr(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    Return(Option<Expr>, Mark),
    /// Ends the path: the helper never returns, so the executor never runs.
    Throw,
    /// `for (const name of iterable) body`, unrolled over a known array.
    ForOf(String, Expr, Vec<Stmt>),
    /// Literal returns and a proven throwing `default`.
    Switch(Expr, Vec<SwitchArm>),
    Opaque,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Entry {
    Prop(String, Expr),
    Spread(Expr),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Expr {
    /// A string literal, kept with its physical positions.
    Lit(RecoveredSql),
    Bool(bool),
    Null,
    Undefined,
    Num,
    Ident(String),
    /// An untagged template: every interpolation must be a literal string.
    Template(Vec<RecoveredSql>, Vec<(Expr, Mark)>),
    /// A trusted-tag template: interpolations are binds or inlined fragments.
    Sql(Vec<RecoveredSql>, Vec<(Expr, Mark)>, bool),
    Member(Box<Expr>, String),
    Object(Vec<Entry>),
    Array(Vec<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Nullish(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    /// `left === right` (strict) or `left == right`, negated for `!==`/`!=`.
    Eq(Box<Expr>, Box<Expr>, bool, bool),
    Add(Box<Expr>, Box<Expr>),
    Append(Box<Expr>, Box<Expr>, Mark),
    Call(String, Vec<Expr>),
    Opaque,
}

#[cfg(test)]
mod tests;
