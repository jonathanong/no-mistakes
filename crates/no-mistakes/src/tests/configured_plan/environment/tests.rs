use super::*;
use crate::tests::TestFramework;

#[test]
fn framework_name_includes_jest() {
    assert_eq!(framework_name(TestFramework::Jest), "jest");
}
