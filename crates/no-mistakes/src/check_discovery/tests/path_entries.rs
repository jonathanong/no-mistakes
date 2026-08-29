#![cfg(unix)]

use super::*;
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn rules_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency")
            .join(name),
    )
}

#[cfg(unix)]
#[test]
fn automatic_check_views_keep_directory_target_symlinks_only_in_inventory() {
    let root = rules_fixture("path-regex-directory-symlink");
    let link = root.join("alpha");
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);

    assert!(sources.inventory().paths().contains(&link));
    let classification = sources
        .inventory()
        .classification_for_path(&link)
        .expect("directory-target symlink is classified");
    assert!(classification.is_lexical_symlink());
    assert!(classification.is_lexical_file() || classification.is_lexical_symlink());
    assert!(!classification.target_is_file());

    let views = discover_check_file_views_from_snapshot(
        &root,
        &NoMistakesConfig::default(),
        &[],
        false,
        &snapshot,
    );
    assert!(!views.filesystem.contains(&link));
    assert!(!views.graph.contains(&link));
}

#[cfg(unix)]
#[test]
fn discovery_does_not_walk_through_directory_target_skill_links() {
    let root = rules_fixture("path-regex-skill-symlinks");
    let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let inventory = sources.inventory();
    let link = root.join(".claude/skills/agent-workflow");
    let through_link = root.join(".claude/skills/agent-workflow/SKILL.md");
    let real_skill = root.join(".agents/skills/agent-workflow/SKILL.md");

    assert!(inventory.paths().contains(&link));
    assert!(inventory.paths().contains(&real_skill));
    assert!(
        !inventory.paths().contains(&through_link),
        "walkers must not descend a directory-target symlink as a tree"
    );
}
