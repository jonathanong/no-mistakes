use super::is_finite_validator;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use std::path::PathBuf;

#[test]
fn member_callees_are_not_finite_validators() {
    let graph = from_raw_maps(
        PathBuf::from("/repo"),
        Default::default(),
        Default::default(),
    );
    let callee = ValueExpression::Member(
        std::sync::Arc::new(ValueExpression::Named {
            name: "schema".into(),
            scope: None,
            offset: 0,
            context: None,
        }),
        "toLowerCase".into(),
    );
    assert!(!is_finite_validator(
        &graph,
        &CheckFactMap::default(),
        &PathBuf::from("/repo/src/query.ts"),
        &callee
    ));
}
