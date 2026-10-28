//! Proven-throw switches, throw guards, and bound `??`.

use super::{exec, fails, lit, load, one, throws};
use crate::codebase::postgres::embedded::helpers::SiteArg;

#[test]
fn switch_returns_every_literal_arm_unless_default_throws() {
    let p = load("exec.ts");
    let posts = "sql:UPDATE posts SET read_at = NOW() WHERE id = $1";
    let comments = "sql:UPDATE comments SET read_at = NOW() WHERE id = $1";
    // Unknown selector: both arms. The throwing default adds nothing.
    assert_eq!(
        exec(&p, "markReadSql", &[SiteArg::Scalar]).unwrap(),
        [posts, comments]
    );
    assert_eq!(one(&p, "markReadSql", &[lit("post")]), posts);
    // A value that matches no arm, including an inherited key, reaches default.
    throws(&p, "markReadSql", &[lit("constructor")]);
    throws(&p, "markReadSql", &[SiteArg::Bool(true)]);

    let owned = "sql:SELECT id FROM orders WHERE account_id = $1";
    let member = "sql:SELECT id FROM orders WHERE member_id = $1";
    let all = "sql:SELECT id FROM orders WHERE account_id = $1 UNION ALL SELECT id FROM orders WHERE member_id = $2";
    assert_eq!(
        exec(&p, "accessSql", &[SiteArg::Scalar]).unwrap(),
        [owned, member, all]
    );
    assert_eq!(one(&p, "accessSql", &[lit("all")]), all);
    throws(&p, "accessSql", &[lit("constructor")]);

    assert_eq!(
        exec(&p, "tableName", &[SiteArg::Scalar]).unwrap(),
        ["str:orders", "str:accounts"]
    );
    assert_eq!(one(&p, "tableName", &[lit("x")]), "str:orders");
    throws(&p, "tableName", &[lit("constructor")]);
    assert_eq!(
        exec(&p, "tableViaThrow", &[SiteArg::Scalar]).unwrap(),
        ["str:orders", "str:accounts"]
    );
    throws(&p, "tableViaThrow", &[lit("constructor")]);
    assert_eq!(
        one(&p, "groupedKind", &[lit("b")]),
        "sql:SELECT 1 FROM posts WHERE id = $1"
    );
    assert_eq!(
        exec(&p, "groupedKind", &[SiteArg::Scalar]).unwrap().len(),
        2
    );
    throws(&p, "groupedKind", &[lit("constructor")]);
    assert_eq!(
        exec(&p, "constThenThrow", &[SiteArg::Scalar]).unwrap(),
        ["sql:CREATE TABLE finite_x (id int)", "sql:SELECT 1"]
    );
    assert_eq!(
        exec(&p, "callToAbrupt", &[SiteArg::Scalar]).unwrap(),
        ["sql:CREATE TABLE finite_x (id int)"]
    );
    throws(&p, "constThenThrow", &[lit("constructor")]);
    throws(&p, "callToAbrupt", &[lit("constructor")]);
    fails(&p, "callCanReturn", &[SiteArg::Scalar]);
    for name in [
        "unsafeDefaults",
        "generatorDefault",
        "optionalDefault",
        "reassignedDefault",
    ] {
        fails(&p, name, &[SiteArg::Scalar]);
    }

    // Not proven: the name assertNever, a local before return, no default
    // (usesSwitch), past the cap, or async.
    let Some(super::super::Stmt::Switch(_, _, Some(super::super::Expr::Call(callee, _)))) = p
        .get("assertNeverIsNotProof")
        .and_then(|program| program.body.first())
    else {
        panic!("expected deferred default call");
    };
    assert_eq!(callee, "assertNever");
    assert!(
        !p.contains_key("assertNever"),
        "returning helper is not abrupt"
    );
    assert_eq!(
        exec(&p, "assertNeverIsNotProof", &[SiteArg::Scalar]),
        None,
        "an unknown selector may reach the returning default"
    );
    // A fixed matching label never reaches the returning default.
    assert_eq!(
        one(&p, "assertNeverIsNotProof", &[lit("x")]),
        "sql:SELECT 1 FROM orders"
    );
    // A repeated label abstains; the first arm is not kept.
    fails(&p, "duplicateLabel", &[lit("a")]);
    // A label that fell through to default is not a later arm.
    fails(&p, "duplicateAfterDefault", &[lit("a")]);
    // A parameter or a parameter of the callee shadows the proven thrower.
    fails(
        &p,
        "shadowedThrowParam",
        &[SiteArg::Scalar, SiteArg::Scalar],
    );
    fails(&p, "shadowedThrowWrapper", &[SiteArg::Scalar]);
    fails(&p, "switchWithLocal", &[lit("a")]);
    fails(&p, "switchCap", &[SiteArg::Scalar]);
    assert_eq!(one(&p, "switchCap", &[lit("01")]), "sql:SELECT 1");
    assert!(!p.contains_key("asyncSwitch"));
    assert_eq!(exec(&p, "failClosed", &[SiteArg::Scalar]), Some(Vec::new()));
}

#[test]
fn opaque_throw_guards_cannot_skip_possible_sql_mutation() {
    let p = load("exec.ts");
    // A name or member-call shape alone does not prove a guard is pure.
    fails(&p, "guardedLimit", &[SiteArg::Scalar]);
    fails(&p, "guardedBlock", &[SiteArg::Scalar]);
    fails(&p, "guardedAnd", &[SiteArg::Scalar, SiteArg::Scalar]);
    fails(&p, "mutatingMethodGuard", &[SiteArg::Scalar]);
    // Both arms return SQL, so an opaque test still fails closed.
    fails(&p, "opaqueBranch", &[SiteArg::Scalar]);
}

#[test]
fn nullish_on_a_bound_scalar_stays_a_bind() {
    let p = load("exec.ts");
    let bound = "sql:SELECT id FROM orders WHERE account_id = $1 AND id > $2";
    // Module const and local const are binds, not SQL text, for an unknown scalar.
    assert_eq!(one(&p, "nullishOuter", &[SiteArg::Scalar]), bound);
    assert_eq!(
        one(&p, "nullishLocal", &[SiteArg::Scalar]),
        "sql:SELECT id FROM orders WHERE id > $1"
    );
    assert_eq!(
        one(&p, "nullishLocal", &[SiteArg::Null]),
        "sql:SELECT id FROM orders WHERE id > $1"
    );
    assert_eq!(
        one(&p, "nullishLocal", &[lit("9")]),
        "sql:SELECT id FROM orders WHERE id > $1"
    );
    // Known nullish with an unbound fallback cannot invent the constant's text.
    fails(&p, "nullishOuter", &[SiteArg::Null]);
    // Free name: Void is not a judged SQL result. The extractor rejects it.
    assert_eq!(exec(&p, "nullishFragment", &[]).unwrap(), ["void"]);
    assert_eq!(
        exec(&p, "nullishSqlFallback", &[SiteArg::Scalar]).unwrap(),
        ["void", "sql:SELECT 1"]
    );
    assert_eq!(
        exec(&p, "nullishNestedFallback", &[SiteArg::Scalar]).unwrap(),
        [
            "sql:SELECT * FROM n14 WHERE a = $1 OFFSET 5",
            "sql:SELECT * FROM n14 WHERE a = SELECT * FROM n14b OFFSET 4 OFFSET 5",
        ]
    );
}
