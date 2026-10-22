use super::{hooks, legacy, Value};
use crate::codebase::postgres::query_annotation::{self, evaluate::concat, module_ids::ModuleIds};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::{path::Path, sync::Arc};

fn saved_function() -> Value {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-shared-function-owner/root.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let mut facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let query_annotation::Expr::Function(summary) = facts.globals.remove("factory").unwrap() else {
        panic!("saved callback summary");
    };
    let ids = ModuleIds::new([path.clone()]);
    Value::Function(summary, ids.shared_path(&path), 7)
}

#[test]
fn real_saved_function_dedup_uses_production_owner_identity_before_content() {
    let value = saved_function();
    let join = || concat::joined(vec![value.clone(), value.clone()]);
    let (actual, work) = hooks::measure_owner(false, join);
    let (old, old_work) = hooks::measure_owner(true, join);
    assert!(legacy::equal(&actual, &old));
    assert!(matches!(actual, Value::Function(..)));
    assert_eq!(work, 0, "shared production owner must skip Path equality");
    assert!(
        old_work > 0,
        "control executes the same production content component"
    );
    let Value::Function(summary, owner, env) = value else {
        unreachable!()
    };
    let independent = Value::Function(summary.clone(), Arc::from(owner.as_ref()), env);
    let shared = Value::Function(summary.clone(), owner.clone(), env);
    let (equal, work) = hooks::measure_owner(false, || shared == independent);
    assert!(equal && work > 0);
    assert!(legacy::equal(&shared, &independent));
    assert!(shared != Value::Function(summary.clone(), owner.clone(), env + 1));
    assert!(
        shared != Value::Function(summary.clone(), Path::new("different-file.mts").into(), env)
    );
    let mut changed = (*summary).clone();
    changed.supported = !changed.supported;
    let different = Value::Function(Arc::new(changed), Arc::from(owner.as_ref()), env);
    assert_eq!(
        hooks::measure_owner(false, || shared == different),
        (false, 0)
    );
}

#[test]
fn lexical_path_equality_retains_the_frozen_pathbuf_oracle() {
    let Value::Function(summary, _, env) = saved_function() else {
        unreachable!()
    };
    for (left, right) in [
        ("a/./b", "a/b"),
        ("a//b/", "a/b"),
        ("./a", "a"),
        ("a/../b", "b"),
    ] {
        let a = Value::Function(summary.clone(), Path::new(left).into(), env);
        let b = Value::Function(summary.clone(), Path::new(right).into(), env);
        assert_eq!(a == b, Path::new(left) == Path::new(right));
        assert_eq!(a == b, legacy::equal(&a, &b));
    }
}

#[test]
fn saved_function_candidates_keep_environment_identity_and_the_sixteen_variant_cap() {
    let Value::Function(summary, owner, _) = saved_function() else {
        unreachable!()
    };
    let values = |count| {
        (0..count)
            .map(|env| Value::Function(summary.clone(), owner.clone(), env))
            .collect()
    };
    let Value::Joined(actual) = concat::joined(values(16)) else {
        panic!("sixteen distinct closures")
    };
    assert_eq!(actual.len(), 16);
    for (env, value) in actual.iter().enumerate() {
        assert!(legacy::equal(
            value,
            &Value::Function(summary.clone(), owner.clone(), env)
        ));
    }
    assert!(concat::joined(values(17)) == Value::Failure("variant-limit"));
    let mut duplicates = values(16);
    duplicates.push(Value::Function(
        summary.clone(),
        Arc::from(owner.as_ref()),
        0,
    ));
    let Value::Joined(deduped) = concat::joined(duplicates) else {
        panic!("equal owner does not overflow")
    };
    assert_eq!(deduped.len(), 16);
}
