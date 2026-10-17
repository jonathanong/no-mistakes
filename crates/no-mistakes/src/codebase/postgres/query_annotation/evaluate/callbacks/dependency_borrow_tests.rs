use super::{Function, Reads, Step};

fn legacy_names(function: &Function, cloned: &mut usize) -> Reads {
    let mut names = Reads::default();
    for step in &function.body {
        match step {
            Step::Bind(_, value)
            | Step::Hoisted(_, value)
            | Step::Effect(value)
            | Step::Return(value) => super::expression(value, &mut names),
            Step::Append(name, value) => {
                names.values.insert(name.clone());
                super::expression(value, &mut names);
            }
            Step::Branch(test, left, right) => {
                super::expression(test, &mut names);
                for body in [left, right] {
                    *cloned += 2; // Both actual owned clone operations below.
                    let nested = Function {
                        body: body.clone(),
                        ..function.clone()
                    };
                    let reads = legacy_names(&nested, cloned);
                    names.values.extend(reads.values);
                    names.identities.extend(reads.identities);
                }
            }
            _ => {}
        }
    }
    let shadowed =
        crate::codebase::postgres::query_annotation::evaluate::calls::scopes::shadow_names(
            function,
        );
    for set in [&mut names.values, &mut names.identities] {
        set.retain(|name| !shadowed.contains(name));
    }
    names.identities.retain(|name| !names.values.contains(name));
    names
}

#[test]
fn saved_nested_profiles_preserve_branch_local_shadowing_without_owned_body_copies() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-callback-profile/nested-shadow.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        crate::codebase::postgres::query_annotation::collect(
            program,
            &source,
            &crate::codebase::postgres::EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let mut total_clones = 0;
    let mut branches = 0;
    for name in ["regular", "arrow", "named"] {
        let crate::codebase::postgres::query_annotation::Expr::Function(function) =
            &facts.globals[name]
        else {
            panic!("saved function {name}");
        };
        let mut cloned = 0;
        let old = legacy_names(function, &mut cloned);
        let production = super::names(function);
        assert!(
            old.values == production.values,
            "production entry complete values {name}"
        );
        assert!(
            old.identities == production.identities,
            "production entry complete identities {name}"
        );
        let mut borrowed = 0;
        let new = super::names_in(function, &function.body, &mut |_| borrowed += 1);
        assert!(old.values == new.values, "whole value reads {name}");
        assert!(
            old.identities == new.identities,
            "whole identity reads {name}"
        );
        assert!(
            new.values.contains("outside"),
            "positive captured read {name}"
        );
        assert!(
            !new.values.contains("parameter"),
            "parameter shadows {name}"
        );
        assert!(
            !new.identities.contains("outside"),
            "value read dominates identity {name}"
        );
        assert!(
            new.identities.contains("identityOnly"),
            "write-only identity {name}"
        );
        assert!(
            !new.values.contains("identityOnly"),
            "write-only is not value {name}"
        );
        assert_eq!(
            new.values.contains("this"),
            name == "arrow",
            "lexical this {name}"
        );
        assert_eq!(
            new.values.contains("arguments"),
            name == "arrow",
            "lexical arguments {name}"
        );
        assert!(
            !new.values.contains("recursive"),
            "self or arm shadow {name}"
        );
        assert_eq!(
            cloned,
            2 * borrowed,
            "each borrowed branch replaces two actual clone operations"
        );
        total_clones += cloned;
        branches += borrowed;
    }
    assert!(
        total_clones > 0 && branches > 0,
        "saved branches exercise the removed work"
    );
}

#[test]
fn saved_expression_shapes_preserve_complete_captured_reads() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-callback-profile/nested-shadow.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        crate::codebase::postgres::query_annotation::collect(
            program,
            &source,
            &crate::codebase::postgres::EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let crate::codebase::postgres::query_annotation::Expr::Function(function) =
        &facts.globals["expressions"]
    else {
        panic!("saved expression profiles");
    };
    let mut cloned = 0;
    let old = legacy_names(function, &mut cloned);
    let new = super::names(function);
    assert!(old.values == new.values, "whole captured value set");
    assert!(
        old.identities == new.identities,
        "whole captured identity set"
    );
    for name in [
        "objectKey",
        "objectValue",
        "objectSpread",
        "optionalTarget",
        "optionalArgument",
        "binaryLeft",
        "binaryRight",
        "appendReceiver",
        "appendArgument",
        "memberReceiver",
        "memberKey",
    ] {
        assert!(new.values.contains(name), "positive projected read {name}");
    }
    for name in ["dynamicObject", "optional", "binary", "appended", "member"] {
        assert!(
            !new.values.contains(name),
            "local binding {name} is not captured"
        );
    }
    assert!(
        new.identities.is_empty(),
        "pure reads do not fabricate write identities"
    );
}
