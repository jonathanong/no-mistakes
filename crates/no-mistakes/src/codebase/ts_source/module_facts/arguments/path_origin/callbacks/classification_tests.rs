use super::*;
use crate::codebase::ts_source::module_facts::arguments::path_origin::callbacks::test_support::with_prepared_fixture;
use oxc_semantic::Semantic;

fn symbol(semantic: &Semantic<'_>, name: &str) -> SymbolId {
    semantic
        .scoping()
        .symbol_ids()
        .find(|&symbol| semantic.scoping().symbol_name(symbol) == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn declaration<'a>(semantic: &Semantic<'a>, name: &str) -> AstKind<'a> {
    semantic.nodes().kind(
        semantic
            .scoping()
            .symbol_declaration(symbol(semantic, name)),
    )
}

#[test]
fn temporary_arguments_follow_each_static_helper_declaration_shape() {
    with_prepared_fixture("callback-classification.mts", |prepared| {
        for helper in [
            "arrowHelper",
            "expressionHelper",
            "declaredHelper",
            "aliasHelper",
        ] {
            prepared.with_projector(|projector| {
                let kind = declaration(projector.semantic, helper);
                assert!(projector.callback_temp_argument(kind, 0, 0), "{helper}");
            });
        }
    });
}

#[test]
fn unsupported_helpers_and_parameter_shapes_do_not_classify_as_temporary() {
    with_prepared_fixture("callback-classification.mts", |prepared| {
        for helper in [
            "scalarHelper",
            "absentHelper",
            "destructuredHelper",
            "unusedHelper",
            "repositoryHelper",
        ] {
            prepared.with_projector(|projector| {
                let kind = declaration(projector.semantic, helper);
                assert!(!projector.callback_temp_argument(kind, 0, 0), "{helper}");
            });
        }
        prepared.with_projector(|projector| {
            let kind = declaration(projector.semantic, "declaredHelper");
            assert!(!projector.callback_temp_argument(kind, 1, 0));
            let kind = declaration(projector.semantic, "declaredCallback");
            assert!(!projector.callback_temp_argument(kind, 0, 0));
        });
    });
}

#[test]
fn type_references_destructuring_and_calls_without_an_argument_are_skipped() {
    with_prepared_fixture("callback-classification.mts", |prepared| {
        prepared.with_projector(|projector| {
            let callback = symbol(projector.semantic, "ignoredCallback");
            assert!(projector
                .semantic
                .scoping()
                .get_resolved_references(callback)
                .any(|reference| reference.is_type()));
            let kind = declaration(projector.semantic, "ignoredHelper");
            assert!(!projector.callback_temp_argument(kind, 0, 0));
        });
    });
}

#[test]
fn alias_cycles_and_long_chains_stop_conservatively_at_the_depth_bound() {
    with_prepared_fixture("callback-classification.mts", |prepared| {
        for helper in ["cycleHelper", "depthHelper"] {
            prepared.with_projector(|projector| {
                let kind = declaration(projector.semantic, helper);
                assert!(projector.callback_temp_argument(kind, 0, 0), "{helper}");
                assert!(projector.remaining > 0, "{helper}");
            });
        }
        prepared.with_projector(|projector| {
            let kind = declaration(projector.semantic, "shortHelper");
            assert!(!projector.callback_temp_argument(kind, 0, 0));
        });
    });
}

#[test]
fn exhausted_reference_budget_is_conservative_and_never_underflows() {
    with_prepared_fixture("callback-classification.mts", |prepared| {
        for remaining in [0, 1] {
            prepared.with_projector(|projector| {
                projector.remaining = remaining;
                let kind = declaration(projector.semantic, "shortHelper");
                assert!(projector.callback_temp_argument(kind, 0, 0));
                assert_eq!(projector.remaining, 0);
            });
        }
    });
}

#[test]
fn prepared_fixture_keeps_projector_state_local_to_each_case() {
    with_prepared_fixture("callback-classification.mts", |prepared| {
        for _ in 0..2 {
            prepared.with_projector(|projector| {
                assert_eq!(projector.remaining, 256);
                assert_eq!(projector.call_offset, 0);
                assert_eq!(projector.immutable.cached_bindings(), 0);
                let helper = symbol(projector.semantic, "arrowHelper");
                assert!(projector.immutable_binding(helper));
                assert_eq!(projector.immutable.cached_bindings(), 1);
                let kind = declaration(projector.semantic, "shortHelper");
                assert!(!projector.callback_temp_argument(kind, 0, 0));
                projector.remaining = 0;
                assert!(projector.callback_temp_argument(kind, 0, 0));
                assert_eq!(projector.remaining, 0);
                projector.call_offset = u32::MAX;
            });
        }
    });
}
