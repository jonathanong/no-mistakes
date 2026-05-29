use super::*;
use crate::config::v2::schema::{Project, ProjectType, RuleDef};

fn fixture(path: &str) -> std::path::PathBuf {
    let mut parts = path.splitn(3, '/');
    let category = parts.next().unwrap_or(path);
    let sub = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        p = p.join(rest);
    }
    crate::codebase::ts_resolver::normalize_path(&p)
}

#[test]
fn target_roots_ignore_unknown_projects() {
    let config = crate::config::v2::NoMistakesConfig::default();
    let rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        projects: vec!["missing".to_string()],
        ..Default::default()
    };

    let roots = target_roots(std::path::Path::new("/repo"), &config, &rule);

    assert!(roots.is_empty());
}

#[test]
fn target_roots_use_workspace_root_for_project_without_root() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config
        .projects
        .insert("backend".to_string(), Project::default());
    let rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        projects: vec!["backend".to_string()],
        ..Default::default()
    };

    let roots = target_roots(std::path::Path::new("/repo"), &config, &rule);

    assert_eq!(roots, vec![std::path::PathBuf::from("/repo")]);
}

#[test]
fn target_roots_infer_nextjs_project_root() {
    let root = fixture("config-v2/nextjs-inferred-root");
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            ..Default::default()
        },
    );
    let rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let roots = target_roots(&root, &config, &rule);

    assert_eq!(roots, vec![root.join("web")]);
}

#[test]
fn target_roots_infer_remix_project_root() {
    let root = fixture("config-v2/remix-inferred-root");
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Remix),
            ..Default::default()
        },
    );
    let rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let roots = target_roots(&root, &config, &rule);

    assert_eq!(roots, vec![root.join("web")]);
}

#[test]
fn target_roots_infer_remix_vite_project_root() {
    let root = fixture("config-v2/remix-vite-inferred-root");
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Remix),
            ..Default::default()
        },
    );
    let rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let roots = target_roots(&root, &config, &rule);

    assert_eq!(roots, vec![root.join("web")]);
}

#[test]
fn target_roots_infer_vitejs_project_root() {
    let root = fixture("config-v2/vitejs-inferred-root");
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Vitejs),
            ..Default::default()
        },
    );
    let rule = RuleDef {
        rule: RUST_MAX_LINES_PER_FILE.to_string(),
        projects: vec!["web".to_string()],
        ..Default::default()
    };

    let roots = target_roots(&root, &config, &rule);

    assert_eq!(roots, vec![root.join("web")]);
}
