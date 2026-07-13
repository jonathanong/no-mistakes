use super::*;
use std::path::Path;
use std::process::Command;

fn initialize_git(root: &Path) {
    for args in [
        &["init", "-q", "--initial-branch=main"][..],
        &["add", "."][..],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn aggregate_ignores_auto_tsconfig_but_honors_an_explicit_one() {
    let dir = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    initialize_git(dir.path());

    let automatic = run_all(dir.path().to_path_buf(), None, None).unwrap();
    assert!(automatic
        .codebase
        .iter()
        .any(|finding| finding.export_name == "AliasedOrigin"));
    assert!(automatic.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS
            && finding.import.as_deref() == Some("@lib/lazy")
            && finding.target.is_none()
    }));
    assert!(automatic.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::REQUIRE_STORYBOOK_STORIES
            && finding.target.as_deref() == Some("src/Button.tsx#Button")
    }));
    assert!(!automatic.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::FORBIDDEN_DEPENDENCIES
            && finding.target.as_deref() == Some("src/forbidden.ts")
    }));
    assert!(
        automatic.queues.is_empty(),
        "an ignored auto-discovered tsconfig must not resolve queue aliases: {:#?}",
        automatic.queues
    );

    let explicit = run_all(
        dir.path().to_path_buf(),
        None,
        Some(Path::new("tsconfig.json").to_path_buf()),
    )
    .unwrap();
    assert!(
        !explicit
            .codebase
            .iter()
            .any(|finding| finding.export_name == "AliasedOrigin"),
        "{:#?}",
        explicit.codebase
    );
    assert!(explicit.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS
            && finding.target.as_deref() == Some("src/lazy.ts")
    }));
    assert!(explicit.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::FORBIDDEN_DEPENDENCIES
            && finding.target.as_deref() == Some("src/forbidden.ts")
    }));
    assert!(!explicit.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::REQUIRE_STORYBOOK_STORIES
            && finding.target.as_deref() == Some("src/Button.tsx#Button")
    }));
    assert!(explicit.queues.iter().any(|finding| {
        finding.kind == "unmatched-producer"
            && finding.file == "enqueue.ts"
            && finding.queue_file.as_deref() == Some("src/queues/emails.ts")
            && finding.job.as_deref() == Some("sendWelcome")
    }));
}

#[test]
fn aggregate_storybook_resolves_visible_package_tsconfigs_independently() {
    let dir = crate::test_support::materialize_gitignore_fixture("storybook-project-tsconfigs");
    initialize_git(dir.path());

    let results = run_all(dir.path().to_path_buf(), None, None).unwrap();
    let storybook = results
        .rules
        .iter()
        .filter(|finding| finding.rule == no_mistakes::codebase::rules::REQUIRE_STORYBOOK_STORIES)
        .collect::<Vec<_>>();

    assert_eq!(storybook.len(), 1, "{storybook:#?}");
    assert_eq!(storybook[0].file, "packages/ignored/src/Button.tsx");
    assert_eq!(
        storybook[0].target.as_deref(),
        Some("src/Button.tsx#Button")
    );
}
