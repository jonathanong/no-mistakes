use super::{matrix_axis_values, SimpleMatrixAxes};
use crate::codebase::workflow_topology::value_primitives::OrderedJson;
use std::collections::BTreeMap;

#[test]
fn matrix_axis_values_formats_each_supported_scalar() {
    let items = [
        OrderedJson::String("linux".to_string()),
        OrderedJson::Number(22.into()),
        OrderedJson::Bool(true),
    ];
    let axes: SimpleMatrixAxes<'_> = BTreeMap::from([("axis", &items[..])]);
    assert_eq!(
        matrix_axis_values(axes).unwrap()["axis"],
        vec!["linux", "22", "true"]
    );
}

#[test]
fn matrix_axis_values_rejects_non_scalar_values() {
    let items = [OrderedJson::Null];
    let axes: SimpleMatrixAxes<'_> = BTreeMap::from([("axis", &items[..])]);
    assert!(matrix_axis_values(axes).is_none());
}
