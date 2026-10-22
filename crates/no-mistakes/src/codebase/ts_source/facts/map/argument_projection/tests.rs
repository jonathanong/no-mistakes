use super::{ArgumentProjection, TsFactMap};
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn fact_map_projection_claims_only_collected_callees() {
    let exact = ArgumentProjection::exact(vec!["selected".into()]);
    let empty = TsFactMap::new();
    assert!(!empty.covers_argument_projection(&exact));

    let no_arguments =
        TsFactMap::from_iter_with_plan([], TsFactPlan::default()).with_argument_projection(&exact);
    assert!(!no_arguments.covers_argument_projection(&exact));

    let arguments = TsFactMap::from_iter_with_plan(
        [],
        TsFactPlan {
            call_arguments: true,
            ..Default::default()
        },
    )
    .with_argument_projection(&exact);
    assert!(arguments.covers_argument_projection(&exact));
    assert!(!arguments.covers_argument_projection(&ArgumentProjection::default()));
}
