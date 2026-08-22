use super::*;
use crate::tests::TestFramework;

#[test]
fn language_phases_use_framework_names() {
    for (framework, discover, select) in [
        (TestFramework::Python, "discover.python", "select.python"),
        (TestFramework::Go, "discover.go", "select.go"),
        (TestFramework::Cargo, "discover.cargo", "select.cargo"),
        (TestFramework::Rails, "discover.rails", "select.rails"),
        (TestFramework::Php, "discover.php", "select.php"),
        (TestFramework::Java, "discover.java", "select.java"),
        (TestFramework::Kotlin, "discover.kotlin", "select.kotlin"),
        (TestFramework::Jest, "discover.jest", "select.jest"),
    ] {
        assert_eq!(discover_phase(framework), discover);
        assert_eq!(select_phase(framework), select);
    }
}
