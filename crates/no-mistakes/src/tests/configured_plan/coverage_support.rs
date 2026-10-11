use super::super::TestFramework;
use no_mistakes::config::v2::schema::TestPlanGroupType;

pub(super) fn reject_unsupported_coverage_group(
    framework: TestFramework,
    group_type: TestPlanGroupType,
) -> anyhow::Result<()> {
    if matches!(
        framework,
        TestFramework::Dotnet
            | TestFramework::Vitest
            | TestFramework::Jest
            | TestFramework::Swift
            | TestFramework::Python
            | TestFramework::Go
            | TestFramework::Cargo
            | TestFramework::Rails
            | TestFramework::Php
            | TestFramework::Java
            | TestFramework::Kotlin
            | TestFramework::Elixir
            | TestFramework::Dart
    ) && group_type == TestPlanGroupType::Coverage
    {
        anyhow::bail!(
            "{} test plans do not support the coverage group",
            super::framework_name(framework)
        );
    }
    Ok(())
}
