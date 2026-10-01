use super::super::{check_source, compile_options, Options};
use super::DEFAULT_PATTERNS;
use std::path::PathBuf;

fn refs(line: &str) -> Vec<String> {
    let options = compile_options(&Options::default()).unwrap();
    check_source("src/app.test.mts", line, &options)
        .into_iter()
        .filter(|finding| finding.target.as_deref() == Some("exact action ref"))
        .filter_map(|finding| finding.import)
        .collect()
}

#[test]
fn the_whole_action_path_is_reported() {
    // The ref is `owner/repo/path@ref`, not its last two components.
    for (line, expected) in [
        ("uses: actions/checkout@v4", "actions/checkout@v4"),
        (
            "uses: github/codeql-action/init@v3",
            "github/codeql-action/init@v3",
        ),
        (
            "uses: github/codeql-action/upload-sarif@v3.25.0",
            "github/codeql-action/upload-sarif@v3.25.0",
        ),
        (
            "- uses: 'github/codeql-action/init@v3'",
            "github/codeql-action/init@v3",
        ),
        (
            "\"uses\": \"github/codeql-action/init@v3\"",
            "github/codeql-action/init@v3",
        ),
        (
            "expect(workflow).toContain('uses: github/codeql-action/init@v3')",
            "github/codeql-action/init@v3",
        ),
        // A reusable workflow path has dot segments and a file name.
        (
            "uses: octo-org/example-repo/.github/workflows/reusable.yml@v1",
            "octo-org/example-repo/.github/workflows/reusable.yml@v1",
        ),
        ("uses: owner/repo/a/b/c/d@v1.2", "owner/repo/a/b/c/d@v1.2"),
        ("uses: owner/repo/a_b/c-d.e@2", "owner/repo/a_b/c-d.e@2"),
    ] {
        assert_eq!(refs(line), [expected], "{line}");
    }
    let sha = "de0fac2e4500dabe0009e67214ff5f5447ce83dd";
    let pinned = format!("github/codeql-action/init@{sha}");
    assert_eq!(
        refs(&format!("uses: {pinned}")),
        std::slice::from_ref(&pinned)
    );
    assert_eq!(
        refs(&format!("uses: {pinned} # v3.25.0")),
        [format!("{pinned} # v3.25.0")]
    );
}

#[test]
fn each_ref_on_a_line_is_reported_whole() {
    assert_eq!(refs("uses: a/b/c@1 uses: d/e/f@2"), ["a/b/c@1", "d/e/f@2"]);
    assert_eq!(refs("a/b@1.2.3/c/d@4.5.6"), ["a/b@1.2.3", "c/d@4.5.6"]);
}

#[test]
fn a_path_that_is_not_a_versioned_ref_is_not_reported() {
    for line in [
        "uses: github/codeql-action/init@main",
        "uses: github/codeql-action/init@release",
        "uses: github/codeql-action/init@",
        "uses: github/codeql-action/init",
        "uses: github/codeql-action/init@v3x",
        // An empty segment is not a path.
        "uses: github/codeql-action//init@v3",
        "uses: owner/repo//path@v1",
        "uses: owner/repo/@v1",
        "uses: owner@v1",
        // A segment with no name in it is not a component of the ref.
        "uses: a/../b@v1",
        "uses: a/./b@v1",
        "uses: a/-/b@v1",
    ] {
        assert!(refs(line).is_empty(), "{line}: {:?}", refs(line));
    }
}

#[test]
fn a_scoped_package_with_a_subpath_is_not_an_action_ref() {
    // `@scope/pkg/sub@1.2.3` starts at `@`, like `@scope/pkg@1.2.3`. Reading its
    // tail as `pkg/sub@1.2.3` would report a package specifier as an action.
    for line in [
        "@scope/pkg/sub@1.2.3",
        "x @scope/pkg/sub@1.2.3 y",
        "'@scope/pkg/sub@1.2.3'",
        "@scope/pkg/deep/sub@1.2.3",
        "foo@bar/baz/qux@1.2.3",
        "@scope/pkg@1.2.3",
        "@actions/checkout@v4",
    ] {
        assert!(refs(line).is_empty(), "{line}: {:?}", refs(line));
    }
    assert_eq!(refs("scope/pkg/sub@1.2.3"), ["scope/pkg/sub@1.2.3"]);
}

#[test]
fn a_path_before_the_ref_is_part_of_the_reported_text() {
    // The ref is the whole `/`-separated path up to the `@`, so a URL host, a Go
    // module path or a directory is kept; a leading `/`, `./` or scheme is not.
    for (line, expected) in [
        (
            "https://github.com/actions/checkout@v4",
            "github.com/actions/checkout@v4",
        ),
        ("golang.org/x/tools@v0.1.0", "golang.org/x/tools@v0.1.0"),
        (
            "github.com/stretchr/testify@v1.8.4",
            "github.com/stretchr/testify@v1.8.4",
        ),
        (
            "https://cdn.jsdelivr.net/npm/pkg@1.2.3/dist/x.js",
            "cdn.jsdelivr.net/npm/pkg@1.2.3",
        ),
        (
            "ls node_modules/.pnpm/pkg@1.2.3/node_modules/pkg",
            "node_modules/.pnpm/pkg@1.2.3",
        ),
        ("/home/runner/x/y@v1", "home/runner/x/y@v1"),
        ("./a/b/c@v1", "a/b/c@v1"),
    ] {
        assert_eq!(refs(line), [expected], "{line}");
    }
}

#[test]
fn a_reserved_host_in_the_path_makes_the_ref_synthetic() {
    // The synthetic exemption judges the whole text. It used to see only the
    // last two components, so `example.com/` in front of them was invisible.
    for line in [
        "https://example.com/actions/checkout@v4",
        "https://api.example.org/a/b@1.2",
        "registry.example/a/b@v1",
        "my.test/actions/checkout@v4",
        "localhost/actions/checkout@v4",
        "a.test/b/c@v1",
    ] {
        assert!(refs(line).is_empty(), "{line}: {:?}", refs(line));
    }
    // Lookalike hosts are real hosts.
    for (line, expected) in [
        (
            "https://github.com/actions/checkout@v4",
            "github.com/actions/checkout@v4",
        ),
        ("example.com.evil.io/a/b@v1", "example.com.evil.io/a/b@v1"),
        ("notexample.com/a/b@v1", "notexample.com/a/b@v1"),
        ("example.io/a/b@v1", "example.io/a/b@v1"),
        // Only the first component is a host; a repo or directory named `b.test`
        // is not.
        ("a/b.test/c@v1", "a/b.test/c@v1"),
        ("x/.test/b@v1", "x/.test/b@v1"),
    ] {
        assert_eq!(refs(line), [expected], "{line}");
    }
}

#[test]
fn a_zero_version_directory_does_not_hide_a_real_ref() {
    // Zero versions are read in the last two path components and the ref, as
    // when only those were reported, so a `v0.0.0` directory above them is a
    // path and the ref after it is still a pin.
    for line in [
        "uses: owner/repo/v0.0.0/path/action@v1",
        "uses: owner/repo/v1.0.0/x/y@v1",
        "uses: a/0.0.0/b/c@v1",
        "uses: a/0.0/b/c@v1.2.3",
        "uses: owner/repo/0.0.0/.github/workflows/x.yml@v1",
    ] {
        let expected = line.strip_prefix("uses: ").unwrap();
        assert_eq!(refs(line), [expected], "{line}");
    }
}

#[test]
fn a_zero_version_in_the_last_two_components_or_the_ref_is_a_placeholder() {
    let sha = "de0fac2e4500dabe0009e67214ff5f5447ce83dd";
    let commented = format!("uses: a/b@{sha} # v0.0.0");
    for line in [
        "uses: owner/repo@v0.0.0",
        "uses: owner/repo@0.0",
        "uses: a/b/c/d/e@0.0.0",
        "uses: a/0.0.0/b@v1",
        "uses: foo-v0.0.0/bar@v1",
        "uses: a/b/foo-v0.0.0/bar@v1",
        "uses: a/b/c@v1 # v0.0.0",
        commented.as_str(),
        // The window starts at the first letter or digit of its first
        // component, so a leading `.` or `-` does not hide the zero version.
        "uses: x/.0.0/b@v1",
        "uses: x/-0.0.0/b@v1",
        "uses: owner/-.0.0/foo@v1",
    ] {
        assert!(refs(line).is_empty(), "{line}: {:?}", refs(line));
    }
    // A real version in the window keeps it a pin, zero versions beside it or not.
    assert_eq!(refs("uses: a/0.0.0/b@v1.2"), ["a/0.0.0/b@v1.2"]);
    assert_eq!(refs("uses: a/b/c@v1.0.0"), ["a/b/c@v1.0.0"]);
}

#[test]
fn the_documented_custom_pattern_is_the_default_action_ref_regex() {
    // The docs show how to replace the default patterns by copying this one.
    let (reason, regex, _) = DEFAULT_PATTERNS[0];
    assert_eq!(reason, "exact action ref");
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/rules/test-no-dependency-pins.md");
    let body = std::fs::read_to_string(docs).unwrap();
    assert!(body.contains(&format!("regex: '{regex}'")), "{regex}");
}
