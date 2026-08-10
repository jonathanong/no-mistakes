use super::*;

#[test]
fn nonboolean_default_truthiness_handles_every_scalar_variant() {
    assert_eq!(default_falsy_state(None), StaticBool::False);
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Bool(false))),
        StaticBool::False
    );
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Bool(true))),
        StaticBool::TruthyNonBoolean
    );
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Number(serde_json::Number::from(0)))),
        StaticBool::False
    );
    assert_eq!(
        default_falsy_state(Some(&JsonScalar::Text(String::new()))),
        StaticBool::False
    );
}
