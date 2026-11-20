use super::super::options::NameFilters;

fn allow() -> Vec<(&'static str, NameFilters)> {
    super::widgets()
}

fn reject() -> Vec<(&'static str, NameFilters)> {
    vec![("items", NameFilters::Reject)]
}

#[test]
fn dynamic_flag_values_stay_judgeable() {
    let file = ["src/widgets.test.ts"];
    for script in [
        "vitest run --project widgets src/widgets.test.ts --shard ${{ matrix.shard }}",
        "vitest run --project widgets src/widgets.test.ts --shard \"${{ matrix.shard }}\"",
        "vitest run --project widgets src/widgets.test.ts --shard \"$CI_SHARD\"",
        "vitest run --project widgets src/widgets.test.ts --config \"$CONFIG\"",
        "vitest run --project widgets src/widgets.test.ts --outputFile \"$OUT\"",
    ] {
        assert_eq!(super::arguments(script, &allow()), file, "{script}");
    }
    assert!(super::arguments(
        "vitest run --project widgets $WIDGETS --shard \"$CI_SHARD\"",
        &allow()
    )
    .is_empty());
    assert!(super::arguments(
        "vitest run --project widgets --custom $VALUE src/widgets.test.ts",
        &allow()
    )
    .is_empty());
    assert!(super::arguments(
        "vitest run --project \"$PROJECT\" src/widgets.test.ts",
        &allow()
    )
    .is_empty());
}

#[test]
fn shell_keywords_and_package_launchers_reach_vitest() {
    let file = ["src/widgets.test.ts"];
    for script in [
        "if true; then vitest run --project widgets src/widgets.test.ts; fi",
        "while true; do vitest run --project widgets src/widgets.test.ts; done",
        "if false; then true; else vitest run --project widgets src/widgets.test.ts; fi",
        "pnpm dlx vitest run --project widgets src/widgets.test.ts",
        "yarn dlx vitest run --project widgets src/widgets.test.ts",
        "pnpm exec ./scripts/with-env vitest run --project widgets src/widgets.test.ts",
        "npx ./scripts/with-env vitest run --project widgets src/widgets.test.ts",
    ] {
        assert_eq!(super::arguments(script, &allow()), file, "{script}");
    }
    assert!(super::arguments(
        "pnpm test -- --project widgets src/widgets.test.ts",
        &allow()
    )
    .is_empty());
    assert!(super::arguments(
        "pnpm run test -- --project widgets src/widgets.test.ts",
        &allow()
    )
    .is_empty());
}

#[test]
fn omitted_projects_and_literal_flag_values_follow_the_policy() {
    assert_eq!(
        super::arguments("vitest run src/widgets.test.ts", &allow()),
        ["src/widgets.test.ts"]
    );
    assert!(super::arguments("vitest run", &allow()).is_empty());
    assert!(
        super::arguments("vitest run --project orders src/orders.test.ts", &allow()).is_empty()
    );
    assert!(super::arguments("vitest run --project items --silent false", &reject()).is_empty());
    assert!(super::arguments(
        "vitest run --project items --coverage.thresholds.lines 80",
        &reject()
    )
    .is_empty());
    assert_eq!(
        super::arguments(
            "vitest run --project items --silent src/items.test.ts",
            &reject()
        ),
        ["src/items.test.ts"]
    );
    assert_eq!(
        super::arguments(
            "vitest run --project items --coverage.thresholds.lines 80 items",
            &reject()
        ),
        ["items"]
    );
}

#[test]
fn numbers_inline_flags_and_a_second_dash_stay_judgeable() {
    let file = ["src/widgets.test.ts"];
    assert!(super::arguments("vitest run --project items --custom 80", &reject()).is_empty());
    assert_eq!(
        super::arguments("vitest run --project items --custom 80 items", &reject()),
        ["items"]
    );
    assert_eq!(
        super::arguments("vitest run --project items --custom 1.5 items", &reject()),
        ["items"]
    );
    assert_eq!(
        super::arguments("vitest run --project items --custom 1.2.3", &reject()),
        ["1.2.3"]
    );
    assert_eq!(
        super::arguments("vitest run --project items --custom .", &reject()),
        ["."]
    );
    assert_eq!(
        super::arguments("vitest run --project items --bail true items", &reject()),
        ["items"]
    );
    assert_eq!(
        super::arguments(
            "vitest run --project widgets -- -- src/widgets.test.ts",
            &allow()
        ),
        file
    );
    assert_eq!(
        super::arguments(
            "vitest run --project widgets --custom=keep src/widgets.test.ts",
            &allow()
        ),
        file
    );
    assert!(super::arguments(
        "vitest run --project widgets --custom=$VALUE src/widgets.test.ts",
        &allow()
    )
    .is_empty());
    assert!(super::arguments(
        "vitest run --project=$WIDGETS src/widgets.test.ts",
        &allow()
    )
    .is_empty());
    assert_eq!(
        super::arguments("vitest run --project= src/widgets.test.ts", &allow()),
        file
    );
    assert_eq!(
        super::arguments(
            "vitest run --project items -t \"$NAME\" src/items.test.ts",
            &reject()
        ),
        ["src/items.test.ts"]
    );
    assert_eq!(
        super::arguments(
            "vitest run --project items --testNamePattern \"$NAME\" items",
            &reject()
        ),
        ["items"]
    );
}
