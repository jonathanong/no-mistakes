use super::*;

#[test]
fn shared_project_array_traversal_keeps_framework_projections_and_safe_fallbacks() {
    let root = fixture("coverage");
    for (framework, file, expected_name, include_suffix) in [
        (
            "playwright",
            "playwright.named-member-projects.ts",
            "pw-named-member-projects",
            "pw-named-member-projects/**/*.spec.ts",
        ),
        (
            "vitest",
            "vitest.named-member-projects.mts",
            "vitest-named-member-projects",
            "vitest-named-member-projects/**/*.test.ts",
        ),
        (
            "playwright",
            "playwright.projects-star-import.ts",
            "pw-projects-star-barrel",
            "pw-projects-star-barrel/**/*.spec.ts",
        ),
        (
            "vitest",
            "vitest.projects-star-import.mts",
            "vitest-projects-star-barrel",
            "vitest-projects-star-barrel/**/*.test.ts",
        ),
    ] {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let projects = if framework == "playwright" {
            parse_playwright_fixture(&source, &path, &root)
                .unwrap()
                .into_projects(&root, file)
        } else {
            parse_vitest_fixture(&source, &path, &root).unwrap()
        };
        assert_eq!(projects.len(), 1, "{file}");
        assert_eq!(
            projects[0].policy_name.as_deref(),
            Some(expected_name),
            "{file}"
        );
        assert!(
            projects[0]
                .include
                .iter()
                .any(|glob| glob.ends_with(include_suffix)),
            "{file}: {:?}",
            projects[0].include,
        );
    }

    // Cycles and unreadable imports leave each config's own default project.
    for (framework, file) in [
        ("playwright", "playwright.projects-star-cycle2.ts"),
        ("vitest", "vitest.projects-star-cycle.mts"),
        ("playwright", "playwright.projects-star-unreadable.ts"),
        ("vitest", "vitest.projects-star-unreadable.mts"),
    ] {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let projects = if framework == "playwright" {
            parse_playwright_fixture(&source, &path, &root)
                .unwrap()
                .into_projects(&root, file)
        } else {
            parse_vitest_fixture(&source, &path, &root).unwrap()
        };
        assert_eq!(projects.len(), 1, "{file}");
        assert!(projects[0].policy_name.is_none(), "{file}");
    }

    let file = "playwright.projects.ts";
    let path = root.join(file);
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_playwright_fixture(&source, &path, &root)
        .unwrap()
        .into_projects(&root, file);
    let names: Vec<_> = projects
        .iter()
        .filter_map(|project| project.policy_name.as_deref())
        .collect();
    let positions: Vec<_> = [
        "imported",
        "imported-spread",
        "factory",
        "wrapped",
        "absolute",
    ]
    .into_iter()
    .map(|name| {
        names
            .iter()
            .position(|candidate| *candidate == name)
            .unwrap()
    })
    .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{names:?}"
    );
}
