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

#[test]
fn root_uses_require_remaining_budget_and_read_only_local_references() {
    with_prepared_fixture("callback-uses.mts", |prepared| {
        for (root, callback, helper, budget, expected) in [
            (
                "budgetRoot",
                "budgetCallback",
                "budgetHelper",
                256,
                Some(()),
            ),
            ("budgetRoot", "budgetCallback", "budgetHelper", 0, None),
            ("writtenRoot", "writeCallback", "writeHelper", 256, None),
            (
                "capturedRoot",
                "closureCallback",
                "closureHelper",
                256,
                None,
            ),
        ] {
            prepared.with_projector(|projector| {
                let root = symbol(projector.semantic, root);
                let callback = symbol(projector.semantic, callback);
                let helper = symbol(projector.semantic, helper);
                let helper = projector
                    .semantic
                    .nodes()
                    .kind(projector.semantic.scoping().symbol_declaration(helper))
                    .span()
                    .start;
                projector.remaining = budget;
                assert_eq!(
                    projector.callback_root_uses(root, callback, false, helper),
                    expected
                );
            });
        }
    });
}
