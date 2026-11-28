use super::*;
use crate::codebase::dependencies::extract::ImportedBindingKind;

#[test]
fn builtin_classifiers_cover_globals_prototypes_and_non_builtins() {
    assert!(global_static_call("UnknownObj", "foo").is_none());
    assert!(global_static_call("Math", "max").unwrap().callbacks.is_empty());
    assert_eq!(global_static_call("Array", "from").unwrap().callbacks, &[1]);
    assert!(global_static_call("JSON", "stringify").unwrap().optional_non_callable);
    assert_eq!(global_static_call("Promise", "try").unwrap().callbacks, &[0]);

    assert!(global_target_callbacks("parseInt").is_none());
    assert!(global_target_callbacks("globalThis.Math.max").is_none());
    assert_eq!(global_target_callbacks("globalThis.Array.from").unwrap().callbacks, &[1]);
    assert_eq!(global_target_callbacks("window.JSON.parse").unwrap().callbacks, &[1]);
    assert_eq!(global_target_callbacks("self.Object.groupBy").unwrap().callbacks, &[1]);
    assert_eq!(global_target_callbacks("global.Promise.try").unwrap().callbacks, &[0]);

    assert_eq!(proto_method_call("map").unwrap().callbacks, &[0]);
    assert_eq!(proto_method_call("then").unwrap().callbacks, &[0, 1]);
    assert!(proto_method_call("replace").unwrap().optional_non_callable);
    assert!(proto_method_call("join").unwrap().callbacks.is_empty());
    assert!(proto_method_call("publish").is_none());

    assert!(is_builtin_constructor("Set"));
    assert!(!is_builtin_constructor("CustomQueue"));

    let mut imports = FxHashMap::default();
    let shadowed = ImportedBinding {
        local: "Boolean".into(),
        imported: "Boolean".into(),
        specifier: "./shadow".into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    };
    let named_fn = ValueExpression::Named {
        name: "Boolean".into(),
        scope: None,
        offset: 0,
        context: None,
    };
    let member_fn = ValueExpression::Member(
        std::sync::Arc::new(ValueExpression::Named {
            name: "Math".into(),
            scope: None,
            offset: 0,
            context: None,
        }),
        "abs".into(),
    );
    assert!(is_builtin_global_fn(&named_fn, &imports));
    assert!(is_builtin_global_fn(&member_fn, &imports));
    let callback_member = |object: &str, method: &str| {
        ValueExpression::Member(
            std::sync::Arc::new(ValueExpression::Named {
                name: object.into(),
                scope: None,
                offset: 0,
                context: None,
            }),
            method.into(),
        )
    };
    assert!(!is_builtin_global_fn(
        &callback_member("Promise", "try"),
        &imports
    ));
    assert!(!is_builtin_global_fn(
        &callback_member("Array", "from"),
        &imports
    ));
    assert!(!is_builtin_global_fn(
        &callback_member("JSON", "parse"),
        &imports
    ));
    assert!(!is_builtin_global_fn(&ValueExpression::Unknown, &imports));
    assert!(!is_builtin_global_fn(
        &ValueExpression::Member(std::sync::Arc::new(ValueExpression::Unknown), "abs".into()),
        &imports
    ));
    assert!(global_target_callbacks("foo.bar").is_none());
    imports.insert("Boolean", &shadowed);
    assert!(!is_builtin_global_fn(&named_fn, &imports));
}
