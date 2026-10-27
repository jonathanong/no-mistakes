use crate::codebase::rules::run_check;

fn messages(case: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    run_check(&super::fixture(case), None, None).unwrap()
}

#[test]
fn same_origin_star_reexports_stay_bound_and_distinct_definitions_do_not() {
    // A direct star and a named re-export of that star are one function.
    // Two definitions, a cycle, and a re-export whose source is missing stay
    // unresolved, which is what fails when every repeated name is dropped.
    let found = messages("shared-star");
    for (file, name) in [
        ("packages/kit/direct.mts", "sharedValue"),
        ("packages/kit/runtime.mts", "pooledRead"),
        ("packages/kit/partial-def.mts", "partial"),
    ] {
        assert!(
            found
                .iter()
                .any(|row| row.file == file && row.target.as_deref() == Some(name)),
            "{name} in {found:?}"
        );
        let needle = format!("unresolved symbol `{name}`");
        assert!(
            found.iter().all(|row| !row.message.contains(&needle)),
            "{needle} in {found:?}"
        );
    }
    for name in ["clash", "ghost", "looped"] {
        let needle = format!("unresolved symbol `{name}`");
        assert!(
            found.iter().any(|row| row.message.contains(&needle)),
            "{needle} in {found:?}"
        );
    }
}

#[test]
fn paths_pattern_owns_the_specifier_when_the_target_path_contains_it() {
    // `marker` maps to `./lib/marker/missing.ts`, so every candidate path
    // contains the specifier text. Ownership is the paths pattern. `loose` is
    // only a workspace package, and `joined` is a baseUrl file.
    let found = messages("paths-shape");
    let alias = found
        .iter()
        .find(|row| row.message.contains("unresolved module selector `marker`"))
        .unwrap_or_else(|| panic!("{found:?}"));
    assert_eq!(alias.file, "app/src/use-both.mts");
    assert!(
        found.iter().all(|row| {
            !row.message.contains("unresolved module selector `loose`")
                && !row.message.contains("unresolved module selector `joined`")
        }),
        "{found:?}"
    );
}
