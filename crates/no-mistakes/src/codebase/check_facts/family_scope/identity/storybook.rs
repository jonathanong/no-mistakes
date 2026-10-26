use super::{applies, configure_family_path_scopes, fixture};
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;

#[test]
fn symbols_scope_unions_storybook_seeds() {
    let root = fixture("rules/require-storybook-stories/fixture/covered");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("storybook fixture config");
    let mut plan = CheckFactPlan {
        symbols: true,
        storybook: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut plan).unwrap();
    assert_eq!(plan.storybook_paths, plan.symbols_paths);
}

#[test]
fn storybook_top_level_include_keeps_configured_story_facts() {
    let root = fixture("check-facts/family-scope-storybook");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("storybook include fixture config");
    let mut scoped = CheckFactPlan {
        react: true,
        symbols: true,
        storybook: true,
        source: true,
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let story = normalize_path(&root.join("stories/button.stories.tsx"));
    assert!(
        applies(&scoped.storybook_paths, &story),
        "configured stories must stay in the Storybook extraction scope"
    );
    let scoped_facts = collect_check_facts(&root, files.clone(), scoped);
    let full_facts = collect_check_facts(
        &root,
        files,
        CheckFactPlan {
            react: true,
            symbols: true,
            storybook: true,
            source: true,
            ..Default::default()
        },
    );
    let scoped_story = format!(
        "{:?}",
        scoped_facts
            .ts
            .get(&story)
            .and_then(|facts| facts.storybook.as_ref())
    );
    let full_story = format!(
        "{:?}",
        full_facts
            .ts
            .get(&story)
            .and_then(|facts| facts.storybook.as_ref())
    );
    assert_ne!(
        scoped_story, "None",
        "scoped collection must extract story facts"
    );
    assert_eq!(scoped_story, full_story);
}
