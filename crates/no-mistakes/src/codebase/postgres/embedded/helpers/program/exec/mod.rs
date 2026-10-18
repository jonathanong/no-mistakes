//! Runs a lowered helper against the abstract arguments of one call.
//!
//! Tests that depend only on known values are decided; tests on unknown
//! scalars fork the path into both alternatives (at most
//! [`MAX_EMBEDDED_SQL_VARIANTS`] live paths). A path that throws ends. Every
//! unsupported construct, unresolvable callee or read of a caller-owned
//! builder fails the whole run, so the call stays `Dynamic`.

mod call;
mod expr;
mod forms;
mod loops;
mod ops;
mod sql;
mod stmt;
mod val;

use super::{Param, Program};
use crate::codebase::postgres::embedded::helpers::SiteArg;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;
use std::path::Path as FsPath;
use val::{empty_sql, Obj, Path, Text, Val};

/// Deepest chain of helper-calls-helper that is followed.
pub(crate) const MAX_CALL_DEPTH: u8 = 8;

/// What a helper call produced on one path.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Ret {
    /// Nothing, or a scalar that is not SQL.
    Void,
    /// A string the helper built, with provenance when it wrote it.
    Str(String, Option<RecoveredSql>),
    /// A fresh builder.
    Sql(RecoveredSql),
    /// The builder the caller passed at this parameter index.
    Param(usize),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Outcome {
    pub(crate) ret: Ret,
    /// Text appended to each parameter that received a builder.
    pub(crate) suffixes: Vec<(usize, RecoveredSql)>,
}

/// Result of asking for a helper by name.
pub(crate) enum Callee {
    /// No program of that name: it may still be an ignorable non-SQL function.
    Unresolved,
    /// A program exists but could not be run for these arguments.
    Failed,
    Done(Vec<Outcome>),
}

/// Looks helpers up in the module being executed (and beyond, for imports).
pub(crate) trait Callees {
    fn call(&mut self, name: &str, args: &[SiteArg]) -> Callee;
}

pub(super) struct Machine<'a> {
    pub(super) stamp: Option<&'a FsPath>,
    pub(super) callees: &'a mut dyn Callees,
    pub(super) params: Vec<String>,
}

/// Every alternative outcome of `program` called with `args`, or `None`.
/// `stamp` is the module recorded as the origin of the program's own text
/// (`None` when it is the executor's own module).
pub(crate) fn run(
    program: &Program,
    args: &[SiteArg],
    stamp: Option<&FsPath>,
    callees: &mut dyn Callees,
) -> Option<Vec<Outcome>> {
    let mut machine = Machine {
        stamp,
        callees,
        params: program.params.iter().map(|p| p.name.clone()).collect(),
    };
    let mut path = Path::new();
    let mut builders = Vec::new();
    for (index, param) in program.params.iter().enumerate() {
        let value = match args.get(index) {
            Some(SiteArg::Builder) => {
                if !param.builder {
                    return None;
                }
                let value = path.new_builder(empty_sql(), Some(index));
                if let Val::Sql(id) = value {
                    builders.push((index, id));
                }
                value
            }
            Some(SiteArg::Undefined) | None => default_value(&mut machine, param, &mut path)?,
            Some(arg) => arg_value(arg),
        };
        path.bind(&param.name, value);
    }
    let paths = machine.block(&program.body, vec![path])?;
    let mut outcomes = Vec::new();
    for path in paths {
        let outcome = machine.outcome(path, &builders)?;
        if !outcomes.contains(&outcome) {
            outcomes.push(outcome);
        }
    }
    (outcomes.len() <= MAX_EMBEDDED_SQL_VARIANTS && !outcomes.is_empty()).then_some(outcomes)
}

fn default_value(machine: &mut Machine<'_>, param: &Param, path: &mut Path) -> Option<Val> {
    let Some(default) = &param.default else {
        return Some(Val::Undef);
    };
    let mut results = machine.eval(default, path.clone())?;
    let (next, value) = results.pop().filter(|_| results.is_empty())?;
    *path = next;
    Some(value)
}

fn arg_value(arg: &SiteArg) -> Val {
    match arg {
        SiteArg::Literal(text) => Val::Str(Text {
            value: text.clone(),
            prov: None,
        }),
        SiteArg::Bool(value) => Val::Bool(*value),
        SiteArg::Null => Val::Null,
        SiteArg::Undefined => Val::Undef,
        SiteArg::Object(props) => Val::Obj(Obj {
            props: props
                .iter()
                .map(|(key, value)| (key.clone(), arg_value(value)))
                .collect(),
            open: false,
        }),
        SiteArg::Array(items) => Val::Arr(items.iter().map(arg_value).collect()),
        SiteArg::Scalar | SiteArg::Builder => Val::Unknown,
    }
}
