use super::StaticBool;

#[test]
fn truthy_nonboolean_values_preserve_expression_semantics() {
    assert_eq!(StaticBool::TruthyNonBoolean.negate(), StaticBool::False);
    assert_eq!(
        StaticBool::TruthyNonBoolean.equals(true),
        StaticBool::Unknown
    );
}
