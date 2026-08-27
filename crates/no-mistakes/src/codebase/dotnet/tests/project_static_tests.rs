use super::super::normalize_path;
use super::super::project_static::parse_project_static;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dotnet-static-project-items/fixture")
}

#[test]
fn merges_default_linked_and_removed_compile_items() {
    let root = normalize_path(&fixture());
    let project = root.join("tests/Linked.Tests/Linked.Tests.csproj");
    let local = root.join("tests/Linked.Tests/LocalTests.cs");
    let removed = root.join("tests/Linked.Tests/RemovedTests.cs");
    let readded = root.join("tests/Linked.Tests/ReaddedTests.cs");
    let built = root.join("tests/Linked.Tests/bin/BuiltTests.cs");
    let generated = root.join("tests/Linked.Tests/obj/GeneratedTests.cs");
    let linked = root.join("src/Shared.cs");
    let source = std::fs::read_to_string(&project).unwrap();
    let facts = parse_project_static(
        &project,
        &source,
        &[
            local.clone(),
            removed.clone(),
            readded.clone(),
            built.clone(),
            generated.clone(),
            linked.clone(),
        ],
    );

    assert_eq!(
        facts.compile_files,
        BTreeSet::from([local, readded, generated, linked])
    );
    assert!(!facts.compile_files.contains(&removed));
    assert!(!facts.compile_files.contains(&built));
    assert_eq!(
        facts.project_references,
        BTreeSet::from([root.join("src/A.csproj"), root.join("src/B.csproj")])
    );
}

#[test]
fn honors_disabled_default_compile_items() {
    let root = normalize_path(&fixture());
    let project = root.join("tests/Explicit.Tests/Explicit.Tests.csproj");
    let local = root.join("tests/Explicit.Tests/ExcludedByDefault.cs");
    let linked = root.join("src/Shared.cs");
    let source = std::fs::read_to_string(&project).unwrap();
    let facts = parse_project_static(&project, &source, &[local.clone(), linked.clone()]);

    assert_eq!(facts.compile_files, BTreeSet::from([linked]));
    assert!(!facts.compile_files.contains(&local));
}

#[test]
fn honors_disabled_default_items() {
    let root = normalize_path(&fixture());
    let project = root.join("tests/AllDefaultsDisabled.Tests/AllDefaultsDisabled.Tests.csproj");
    let local = root.join("tests/AllDefaultsDisabled.Tests/ExcludedByDefault.cs");
    let source = std::fs::read_to_string(&project).unwrap();
    let facts = parse_project_static(&project, &source, std::slice::from_ref(&local));

    assert!(facts.compile_files.is_empty());
}

#[test]
fn uses_the_last_compile_specific_default_override() {
    let root = normalize_path(&fixture());
    let project = root.join("tests/CompileOverride.Tests/CompileOverride.Tests.csproj");
    let local = root.join("tests/CompileOverride.Tests/IncludedByOverride.cs");
    let source = std::fs::read_to_string(&project).unwrap();
    let facts = parse_project_static(&project, &source, std::slice::from_ref(&local));

    assert_eq!(facts.compile_files, BTreeSet::from([local]));
}

#[test]
fn keeps_legacy_project_compile_items_explicit() {
    let root = normalize_path(&fixture());
    let project = root.join("tests/Legacy.Tests/Legacy.Tests.csproj");
    let local = root.join("tests/Legacy.Tests/NotExplicitlyCompiled.cs");
    let linked = root.join("src/Shared.cs");
    let source = std::fs::read_to_string(&project).unwrap();
    let facts = parse_project_static(&project, &source, &[local.clone(), linked.clone()]);

    assert_eq!(facts.compile_files, BTreeSet::from([linked]));
    assert!(!facts.compile_files.contains(&local));
}
