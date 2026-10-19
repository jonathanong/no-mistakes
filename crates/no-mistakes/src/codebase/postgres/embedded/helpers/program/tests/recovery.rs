//! Proven-throw switches, throw guards, and bound `??`.

use super::{exec, fails, lit, load, one};
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
    fails(&p, "markReadSql", &[lit("constructor")]);
    fails(&p, "markReadSql", &[SiteArg::Bool(true)]);

    let owned = "sql:SELECT id FROM orders WHERE account_id = $1";
    let member = "sql:SELECT id FROM orders WHERE member_id = $1";
    let all = "sql:SELECT id FROM orders WHERE account_id = $1 UNION ALL SELECT id FROM orders WHERE member_id = $2";
    assert_eq!(
        exec(&p, "accessSql", &[SiteArg::Scalar]).unwrap(),
        [owned, member, all]
    );
    assert_eq!(one(&p, "accessSql", &[lit("all")]), all);
    fails(&p, "accessSql", &[lit("constructor")]);

    assert_eq!(
        exec(&p, "tableName", &[SiteArg::Scalar]).unwrap(),
        ["str:orders", "str:accounts"]
    );
    assert_eq!(one(&p, "tableName", &[lit("x")]), "str:orders");
    fails(&p, "tableName", &[lit("constructor")]);
    assert_eq!(
        exec(&p, "tableViaThrow", &[SiteArg::Scalar]).unwrap(),
        ["str:orders", "str:accounts"]
    );
    assert_eq!(
        one(&p, "groupedKind", &[lit("b")]),
        "sql:SELECT 1 FROM posts WHERE id = $1"
    );
    assert_eq!(
        exec(&p, "groupedKind", &[SiteArg::Scalar]).unwrap().len(),
        2
    );

    // Not proven: the name assertNever, a local before return, no default
    // (usesSwitch), past the cap, or async.
    fails(&p, "assertNeverIsNotProof", &[SiteArg::Scalar]);
    // A known arm is not kept when default is not proven to throw.
    fails(&p, "assertNeverIsNotProof", &[lit("x")]);
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
    assert!(!p.contains_key("failClosed"));
}

#[test]
fn throw_guards_are_skipped_when_they_cannot_write_sql() {
    let p = load("exec.ts");
    assert_eq!(
        one(&p, "guardedLimit", &[SiteArg::Scalar]),
        "sql:SELECT id FROM orders WHERE account_id = $1 LIMIT $2"
    );
    assert_eq!(
        one(&p, "guardedBlock", &[SiteArg::Scalar]),
        "sql:SELECT id FROM orders WHERE account_id = $1"
    );
    assert_eq!(
        one(&p, "guardedAnd", &[SiteArg::Scalar, SiteArg::Scalar]),
        "sql:SELECT id FROM orders WHERE account_id = $1"
    );
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
