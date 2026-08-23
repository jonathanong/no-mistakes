// no-mistakes-disable-file rust-max-lines-per-file: legacy fixture-heavy test module
use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::storybook::StorybookFileFacts;
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig};
use crate::codebase::ts_symbols::{Export, ExportKind, FileSymbols};
use crate::config::v2::schema::{Project, ProjectType, RuleDef, StringOrList};
use crate::react_traits::report::types::{ComponentFacts, ComponentRef, Environment};
use std::collections::HashSet;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/require-storybook-stories/fixture")
        .join(name)
}

fn config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some(".".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        projects: vec!["web".to_string()],
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn config_with_storybook(options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.tests.storybook.configs = Some(StringOrList::One(".storybook/main.ts".to_string()));
    config
}

fn config_with_project_root(root: &str, options: &str) -> NoMistakesConfig {
    let mut config = config(options);
    config.projects.get_mut("web").unwrap().root = Some(root.to_string());
    config
}

fn empty_resolver(root: &std::path::Path) -> ImportResolver<'static> {
    let tsconfig = Box::leak(Box::new(TsConfig {
        dir: root.to_path_buf(),
        paths: vec![],
        paths_dir: root.to_path_buf(),
        base_url: None,
    }));
    ImportResolver::new(tsconfig)
}

#[test]
fn pass4b_storybook_import_skips_ignored_component_for_visible_fallback() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass4b-shadow");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = normalize_path(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .map(|path| normalize_path(&path))
        .collect::<crate::fx::PathSet>();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);

    assert_eq!(
        resolver.resolve("./Button", &root.join("storybook/Button.stories.ts")),
        Some(root.join("storybook/Button.ts"))
    );
}

fn react_component(name: &str, file: &str, children: Vec<ComponentRef>) -> ComponentFacts {
    ComponentFacts {
        name: name.to_string(),
        file: file.to_string(),
        environment: Environment::Client,
        has_state: false,
        has_props: false,
        passes_props: false,
        uses_memo: false,
        uses_context_provider: false,
        uses_suspense: false,
        fetches: Vec::new(),
        dependencies: Vec::new(),
        children,
        inherited_from_children: None,
    }
}

#[test]
fn deferred_suppression_sources_use_prepared_component_text() {
    let root = fixture("comments");
    let component = types::Component {
        key: "components/DisabledFile.tsx#DisabledFile".to_string(),
        file: PathBuf::from("components/DisabledFile.tsx"),
        repo_file: "components/DisabledFile.tsx".to_string(),
        project_file: "components/DisabledFile.tsx".to_string(),
        export_name: "DisabledFile".to_string(),
        line: 2,
        explicit: true,
    };
    let path = normalize_path(&root.join(&component.file));
    let prepared_without_directive = CheckFactMap {
        ts: crate::codebase::ts_source::FileIdMap::from([(
            path.clone(),
            std::sync::Arc::new(CheckFileFacts {
                source: Some("export function DisabledFile() { return <div />; }".into()),
                ..Default::default()
            }),
        )])
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let indexed = suppression::component_suppression_sources(
        &root,
        std::slice::from_ref(&component),
        &prepared_without_directive,
    );
    // The fixture on disk is disabled, but the prepared source is authoritative.
    assert!(!suppression::component_is_suppressed(
        &root, &indexed, &component,
    ));

    let prepared_with_directive = CheckFactMap {
        ts: crate::codebase::ts_source::FileIdMap::from([(
            path,
            std::sync::Arc::new(CheckFileFacts {
                source: Some(
                    "// no-mistakes-disable-file require-storybook-stories: prepared exemption\n\
                     export function DisabledFile() { return <div />; }"
                        .into(),
                ),
                ..Default::default()
            }),
        )])
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let indexed = suppression::component_suppression_sources(
        &root,
        std::slice::from_ref(&component),
        &prepared_with_directive,
    );

    assert!(suppression::component_is_suppressed(
        &root, &indexed, &component,
    ));
}

#[test]
fn missing_project_target_is_ignored() {
    let root = fixture("comments");
    let mut config = config("");
    config.projects.remove("web");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);

    let findings = super::runner::check_with_resolver(
        &root,
        &config,
        &CheckFactMap::default(),
        &empty_resolver(&root),
        None,
        false,
        &sources,
    )
    .unwrap();

    assert!(findings.is_empty());
}

fn react_facts(
    components: Vec<ComponentFacts>,
) -> std::sync::Arc<crate::react_traits::analyze::file::FileAnalysis> {
    crate::react_traits::analyze::file::FileAnalysis {
        components: components.into(),
    }
    .into()
}

mod config_helpers;
mod coverage_helpers;
mod coverage_rule_cases;
mod selection_rule_cases;
