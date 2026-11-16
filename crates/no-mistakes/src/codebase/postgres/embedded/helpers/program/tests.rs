//! Executes lowered helper programs from saved fixtures.

use super::Program;
use super::exec::{Callee, Callees, MAX_CALL_DEPTH, Outcome, Ret, run};
use crate::codebase::postgres::embedded::helpers::SiteArg;
use crate::codebase::postgres::embedded::{
    EmbeddedSqlOptions, INTERNAL_PLACEHOLDER, extract_embedded_sql_from_source,
};
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[path = "tests/lowering.rs"]
mod lowering;
#[path = "tests/raw_sql.rs"]
mod raw_sql;
#[path = "tests/recovery.rs"]
mod recovery;
#[path = "tests/values.rs"]
mod values;

pub(super) type Programs = FxHashMap<String, Arc<Program>>;

pub(super) fn load(name: &str) -> Programs {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/programs")
        .join(name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
            crate::codebase::postgres::embedded::TrustedSqlTag {
                module: "@example/db".into(),
                name: "sql".into(),
            },
        ]),
    )
    .exported_programs
}

/// Resolves callees among the programs of one module, like a projection
/// that never leaves the module.
struct Module<'a> {
    programs: &'a Programs,
    depth: u8,
}

impl Callees for Module<'_> {
    fn call(&mut self, name: &str, args: &[SiteArg]) -> Callee {
        if self.depth >= MAX_CALL_DEPTH {
            return Callee::Failed;
        }
        let Some(program) = self.programs.get(name) else {
            return Callee::Unresolved;
        };
        let mut child = Module {
            programs: self.programs,
            depth: self.depth + 1,
        };
        match run(program, args, None, &mut child) {
            Some(outcomes) => Callee::Done(outcomes),
            None => Callee::Failed,
        }
    }
}

pub(super) fn lit(text: &str) -> SiteArg {
    SiteArg::Literal(text.into())
}

pub(super) fn object(props: &[(&str, SiteArg)]) -> SiteArg {
    SiteArg::Object(
        props
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    )
}

fn published(text: &str) -> String {
    text.replace(INTERNAL_PLACEHOLDER, "$").replace(
        crate::codebase::postgres::embedded::placeholders::INTERNAL_BIND,
        "$",
    )
}

fn render(outcome: &Outcome) -> String {
    let ret = match &outcome.ret {
        Ret::Void => "void".to_string(),
        Ret::Str(value, _) => format!("str:{value}"),
        Ret::Sql(text) => format!("sql:{}", published(&text.sql)),
        Ret::Param(index) => format!("param{index}"),
    };
    let suffixes: Vec<String> = outcome
        .suffixes
        .iter()
        .map(|(index, text)| format!("+{index}:{}", published(&text.sql)))
        .collect();
    format!("{ret}{}", suffixes.join(""))
}

/// Every alternative outcome of `name(args)`, rendered; `None` when it fails
/// or when the function never lowered to a program.
pub(super) fn exec(programs: &Programs, name: &str, args: &[SiteArg]) -> Option<Vec<String>> {
    let program = programs.get(name)?;
    let mut module = Module { programs, depth: 0 };
    run(program, args, None, &mut module).map(|outcomes| outcomes.iter().map(render).collect())
}

fn one(programs: &Programs, name: &str, args: &[SiteArg]) -> String {
    let mut rendered = exec(programs, name, args).unwrap_or_else(|| panic!("{name} failed"));
    assert_eq!(rendered.len(), 1, "{name}: {rendered:?}");
    rendered.remove(0)
}

fn fails(programs: &Programs, name: &str, args: &[SiteArg]) {
    assert_eq!(
        exec(programs, name, args),
        None,
        "{name} should fail closed"
    );
}

fn throws(programs: &Programs, name: &str, args: &[SiteArg]) {
    assert_eq!(
        exec(programs, name, args),
        Some(Vec::new()),
        "{name} should have no returning path"
    );
}

#[path = "tests/coverage_gap.rs"]
mod coverage_gap;
#[path = "switch_boundary_fixture_tests.rs"]
mod switch_boundary_fixture_tests;

#[path = "tests_cases/part1.rs"]
mod part1;

#[path = "tests_cases/part2.rs"]
mod part2;
