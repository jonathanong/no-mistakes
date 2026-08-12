use super::{static_json::static_json_value, StaticValue};
use serde_json::json;

#[test]
fn static_json_values_cover_scalars_sequences_and_conservative_states() {
    for (value, expected) in [
        (StaticValue::Bool(true), Ok(Some(json!(true)))),
        (
            StaticValue::String("release".into()),
            Ok(Some(json!("release"))),
        ),
        (StaticValue::Number("0xff".into()), Ok(Some(json!(255)))),
        (StaticValue::Null, Ok(Some(serde_json::Value::Null))),
        (
            StaticValue::Sequence(vec![StaticValue::Bool(true), StaticValue::Null]),
            Ok(Some(json!([true, null]))),
        ),
        (StaticValue::Invalid, Err(())),
        (StaticValue::Mapping, Ok(None)),
        (StaticValue::MatrixMapping("matrix.cfg".into()), Ok(None)),
        (StaticValue::NonStringable, Ok(None)),
        (StaticValue::Unknown, Ok(None)),
    ] {
        assert_eq!(static_json_value(&value), expected, "{value:?}");
    }
    assert_eq!(
        static_json_value(&StaticValue::Number("invalid".into())),
        Err(())
    );
    assert_eq!(
        static_json_value(&StaticValue::Sequence(vec![StaticValue::Unknown])),
        Ok(None)
    );
    assert_eq!(
        static_json_value(&StaticValue::Sequence(vec![StaticValue::Invalid])),
        Err(())
    );
}
