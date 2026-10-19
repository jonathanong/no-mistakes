//! Executes lowered helper programs from saved fixtures.

use super::exec::{run, Callee, Callees, Outcome, Ret, MAX_CALL_DEPTH};
use super::Program;
use crate::codebase::postgres::embedded::helpers::SiteArg;
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, INTERNAL_PLACEHOLDER,
};
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[path = "tests/lowering.rs"]
mod lowering;
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
        &EmbeddedSqlOptions::configured("@example/db", &[]),
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
    text.replace(INTERNAL_PLACEHOLDER, "$")
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

#[test]
fn strings_and_fresh_builders() {
    let p = load("exec.ts");
    assert_eq!(one(&p, "fresh", &[SiteArg::Scalar]), "sql:SELECT $1");
    assert_eq!(one(&p, "rawSplice", &[lit("p")]), "str:p.x IS NULL");
    fails(&p, "rawSplice", &[SiteArg::Scalar]);
    fails(&p, "rawSplice", &[lit("two\nlines")]);
    fails(&p, "rawSplice", &[SiteArg::Bool(true)]);
    assert_eq!(one(&p, "literalString", &[]), "str:a = 1");
    assert_eq!(
        one(&p, "concatenated", &[lit("a"), lit("b")]),
        "str:a AND b"
    );
    // Nothing in it can contribute SQL, so it never lowers.
    assert!(!p.contains_key("numbers"));
}

#[test]
fn builder_parameters_accumulate_a_suffix() {
    let p = load("exec.ts");
    let b = SiteArg::Builder;
    assert_eq!(
        one(&p, "appender", std::slice::from_ref(&b)),
        "void+0: AND a = 1"
    );
    assert_eq!(
        one(&p, "returnsBuilder", std::slice::from_ref(&b)),
        "param0+0: AND b = 1"
    );
    assert_eq!(
        one(&p, "aliasOfParameter", std::slice::from_ref(&b)),
        "param0+0: AND c = 1"
    );
    assert_eq!(
        one(&p, "inlinesFragment", &[b.clone(), SiteArg::Scalar]),
        "void+0: AND b = $1"
    );
    // A caller-owned builder can be appended to but never read back.
    fails(&p, "readsBuilderParameter", std::slice::from_ref(&b));
    fails(&p, "interpolatesBuilderParameter", std::slice::from_ref(&b));
    // A builder needs a parameter annotated as the SQLStatement type.
    fails(&p, "appendsToNonBuilder", std::slice::from_ref(&b));
    fails(&p, "appendsToNonBuilder", &[lit("x")]);
    fails(&p, "appendsNumber", &[b]);
    fails(&p, "interpolatesFreeName", &[]);
}

#[test]
fn unknown_tests_fork_and_known_ones_decide() {
    let p = load("exec.ts");
    let both = exec(&p, "branches", &[SiteArg::Scalar]).unwrap();
    assert_eq!(
        both,
        ["sql:SELECT 1 WHERE a = 1", "sql:SELECT 1 WHERE b = 1"]
    );
    assert_eq!(
        one(&p, "branches", &[SiteArg::Bool(true)]),
        "sql:SELECT 1 WHERE a = 1"
    );
    assert_eq!(
        one(&p, "branches", &[SiteArg::Bool(false)]),
        "sql:SELECT 1 WHERE b = 1"
    );
    assert_eq!(one(&p, "branches", &[lit("")]), "sql:SELECT 1 WHERE b = 1");
    assert_eq!(one(&p, "branches", &[lit("x")]), "sql:SELECT 1 WHERE a = 1");
    assert_eq!(
        one(&p, "branches", &[SiteArg::Null]),
        "sql:SELECT 1 WHERE b = 1"
    );
    assert_eq!(
        one(&p, "branches", &[object(&[])]),
        "sql:SELECT 1 WHERE a = 1"
    );
    // The same unknown test decides the same way each time it is asked.
    let x = object(&[("x", SiteArg::Scalar)]);
    assert_eq!(
        exec(&p, "repeatedTest", &[x]).unwrap(),
        ["sql:SELECT 1 AND a = 1 AND b = 1", "sql:SELECT 1 AND c = 1"]
    );
}

#[test]
fn comparisons_and_logic_over_known_values() {
    let p = load("exec.ts");
    let known = |x, y| object(&[("x", x), ("y", y)]);
    assert_eq!(
        one(&p, "negatedTests", &[known(SiteArg::Bool(true), lit("z"))]),
        "sql:SELECT 1 AND b = 1 AND c = 1 AND d = 1"
    );
    assert_eq!(
        one(
            &p,
            "negatedTests",
            &[known(SiteArg::Undefined, SiteArg::Null)]
        ),
        "sql:SELECT 1 AND a = 1"
    );
    assert_eq!(
        one(&p, "negatedTests", &[SiteArg::Undefined]),
        "sql:SELECT 1 AND a = 1"
    );
    // Unknown values leave both outcomes of each comparison open.
    assert!(exec(&p, "negatedTests", &[SiteArg::Scalar]).unwrap().len() > 1);
    let t = SiteArg::Bool(true);
    let f = SiteArg::Bool(false);
    assert_eq!(
        one(&p, "logicalTests", &[t.clone(), f.clone()]),
        "sql:SELECT 1 AND either = 1"
    );
    assert_eq!(
        one(&p, "logicalTests", &[f.clone(), f.clone()]),
        "sql:SELECT 1"
    );
    assert_eq!(
        one(&p, "logicalTests", &[t.clone(), t]),
        "sql:SELECT 1 AND both = 1 AND either = 1"
    );
    assert_eq!(
        exec(&p, "logicalTests", &[SiteArg::Scalar, f])
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn logic_as_values_and_statements() {
    let p = load("exec.ts");
    let s = SiteArg::Scalar;
    assert!(exec(&p, "valueLogic", &[s.clone(), s.clone()]).is_some());
    assert!(exec(&p, "valueLogic", &[SiteArg::Null, lit("x")]).is_some());
    assert!(exec(&p, "valueLogic", &[lit("x"), SiteArg::Undefined]).is_some());
    let b = SiteArg::Builder;
    // The same unknown flag decides both statements the same way.
    assert_eq!(
        exec(&p, "statementLogic", &[SiteArg::Scalar, b]).unwrap(),
        ["void+1: AND a = 1 AND b = 1", "void+1: AND c = 1"]
    );
}

#[test]
fn ternary_text_and_aliases() {
    let p = load("exec.ts");
    assert_eq!(
        exec(&p, "ternaryText", &[SiteArg::Scalar, lit("p")]).unwrap(),
        ["sql:SELECT 1 WHERE p.a = 1", "sql:SELECT 1 WHERE p.b = 1"]
    );
    assert_eq!(
        one(&p, "ternaryText", &[SiteArg::Bool(false), lit("p")]),
        "sql:SELECT 1 WHERE p.b = 1"
    );
    fails(&p, "ternaryText", &[SiteArg::Scalar, SiteArg::Scalar]);
}

#[test]
fn throwing_ends_a_path() {
    let p = load("exec.ts");
    assert_eq!(one(&p, "throwsOnUnsafe", &[lit("ok")]), "sql:SELECT 1");
    fails(&p, "throwsOnUnsafe", &[lit("bad")]);
    assert_eq!(
        exec(&p, "throwsOnUnsafe", &[SiteArg::Scalar]).unwrap(),
        ["sql:SELECT 1"]
    );
    assert!(!p.contains_key("alwaysThrows"));
}

#[test]
fn returns_and_blocks() {
    let p = load("exec.ts");
    assert_eq!(
        exec(&p, "earlyReturn", &[SiteArg::Scalar]).unwrap(),
        ["sql:SELECT 1", "sql:SELECT 1 WHERE a = 1"]
    );
    assert_eq!(
        exec(&p, "nestedBlock", &[SiteArg::Scalar]).unwrap(),
        ["sql:SELECT 1 WHERE a = 1", "sql:SELECT 1 WHERE b = 1"]
    );
}

#[test]
fn defaults_apply_to_missing_and_undefined_arguments() {
    let p = load("exec.ts");
    let strict = "sql:SELECT 1 WHERE strict = 1 /* x! */";
    assert_eq!(one(&p, "withDefault", &[lit("x")]), strict);
    assert_eq!(
        one(&p, "withDefault", &[lit("x"), SiteArg::Undefined]),
        strict
    );
    assert_eq!(
        one(&p, "withDefault", &[lit("x"), SiteArg::Bool(false)]),
        "sql:SELECT 1 /* x! */"
    );
    assert_eq!(
        one(
            &p,
            "withDefault",
            &[lit("x"), SiteArg::Bool(false), lit("y")]
        ),
        "sql:SELECT 1 /* y */"
    );
}

#[test]
fn objects_arrays_and_members() {
    let p = load("exec.ts");
    let a = object(&[("a", lit("v"))]);
    assert_eq!(
        one(&p, "members", std::slice::from_ref(&a)),
        "sql:SELECT 1 /* v */"
    );
    fails(&p, "members", &[]);
    fails(&p, "members", &[SiteArg::Scalar]);
    assert_eq!(
        one(&p, "memberOfLiteral", &[object(&[("table", lit("t"))])]),
        "sql:SELECT 1 FROM t"
    );
    fails(
        &p,
        "memberOfLiteral",
        &[object(&[("table", SiteArg::Scalar)])],
    );
    fails(&p, "memberOfLiteral", &[object(&[])]);
    assert_eq!(
        one(&p, "memberOfUnknown", &[SiteArg::Scalar]),
        "sql:SELECT $1, $2"
    );
    fails(&p, "memberOfString", &[lit("abc")]);
    assert_eq!(
        one(&p, "spreads", std::slice::from_ref(&a)),
        "sql:SELECT 1x"
    );
    assert_eq!(one(&p, "spreadsNothing", &[object(&[])]), "sql:SELECT 1z");
    assert_eq!(
        one(&p, "spreadsNothing", &[SiteArg::Scalar]),
        "sql:SELECT 1z"
    );
    fails(&p, "spreadsString", &[]);
    fails(&p, "computedKey", &[]);
    assert_eq!(
        one(&p, "arrayLength", &[SiteArg::Array(vec![lit("a")])]),
        "sql:SELECT $1"
    );
    assert_eq!(
        one(
            &p,
            "arrayElementsAreBound",
            &[SiteArg::Array(vec![lit("a")])]
        ),
        "sql:SELECT $1"
    );
}

#[test]
fn loops_unroll_over_known_arrays() {
    let p = load("exec.ts");
    let two = SiteArg::Array(vec![lit("a"), lit("b")]);
    assert_eq!(
        one(&p, "sumOver", std::slice::from_ref(&two)),
        "sql:SELECT 1 UNION SELECT 1 FROM a UNION SELECT 1 FROM b"
    );
    assert_eq!(
        one(&p, "sumOver", &[SiteArg::Array(vec![])]),
        "sql:SELECT 1"
    );
    fails(&p, "sumOver", &[SiteArg::Scalar]);
    fails(&p, "sumOver", &[SiteArg::Array(vec![lit("a"); 17])]);
    assert_eq!(one(&p, "returnsInsideLoop", &[two]), "sql:SELECT 1 FROM a");
    assert_eq!(
        one(&p, "returnsInsideLoop", &[SiteArg::Array(vec![])]),
        "sql:SELECT 0"
    );
    fails(&p, "loopOverUnknown", &[SiteArg::Scalar]);
}

#[test]
fn calls_between_helpers() {
    let p = load("exec.ts");
    assert_eq!(
        one(&p, "callsFresh", &[SiteArg::Scalar]),
        "sql:WITH x AS (SELECT $1)"
    );
    assert_eq!(
        one(&p, "callsAppender", &[SiteArg::Builder]),
        "param0+0: AND a = 1"
    );
    assert_eq!(
        one(&p, "callsReturningBuilder", &[SiteArg::Builder]),
        "param0+0: AND b = 1"
    );
    // An unresolved statement call with plain arguments cannot change SQL.
    assert_eq!(
        one(&p, "callsUnresolvedStatement", &[lit("a")]),
        "sql:SELECT 1"
    );
    fails(&p, "callsUnresolvedValue", &[lit("a")]);
    fails(&p, "callsUnresolvedWithObject", &[SiteArg::Builder]);
    assert_eq!(
        one(&p, "passesObjectsAndArrays", &[lit("v")]),
        "sql:SELECT 1v"
    );
    fails(&p, "passesBuilderInObject", &[SiteArg::Builder]);
    fails(&p, "callsThroughParameter", &[SiteArg::Scalar]);
    fails(&p, "callsFailing", &[]);
    fails(&p, "selfRecursive", &[]);
}

#[test]
fn unsupported_statements_fail_only_when_reached() {
    let p = load("exec.ts");
    for name in ["usesAssignment", "opaqueTaken", "usesSwitch"] {
        fails(&p, name, &[lit("a")]);
    }
    // These keep their SQL inside statements the model does not know, so
    // there is nothing left to run.
    for name in ["usesTry", "usesVar"] {
        assert!(!p.contains_key(name), "{name}");
    }
    assert_eq!(one(&p, "untaken", &[SiteArg::Bool(false)]), "sql:SELECT 1");
    fails(&p, "untaken", &[SiteArg::Bool(true)]);
    assert_eq!(one(&p, "usesAwait", &[]), "sql:SELECT 1");
    fails(&p, "usesTemplateTag", &[]);
    fails(&p, "usesRawTag", &[]);
    fails(&p, "usesDestructuring", &[SiteArg::Scalar]);
    assert!(!p.contains_key("emptyReturn"));
    assert_eq!(
        one(&p, "voidBody", &[SiteArg::Builder]),
        "void+0: AND a = 1"
    );
}
