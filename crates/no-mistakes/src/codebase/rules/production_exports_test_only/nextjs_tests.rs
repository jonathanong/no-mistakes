use super::nextjs::{is_entrypoint, project_roots};
use super::tests::fixture;
use crate::codebase::rules::run_check;
use crate::codebase::unique_exports::is_framework_entrypoint_file;
use crate::config::v2::schema::{NoMistakesConfig, Project, ProjectType};
use std::path::{Path, PathBuf};

#[test]
fn nextjs_entrypoints_are_production_consumers_and_other_files_are_not() {
    let found = run_check(&fixture("nextjs-entrypoints"), None, None).unwrap();
    let mut actual: Vec<_> = found
        .iter()
        .map(|row| {
            format!(
                "{} {}",
                row.file,
                row.target.as_deref().unwrap_or("missing")
            )
        })
        .collect();
    actual.sort();
    // Entrypoint exports imported only by tests are absent. `fromPage` is
    // imported by `app/page.tsx`, which stays a production consumer.
    assert_eq!(
        actual,
        [
            "other/app/page.tsx otherPage",
            "web/app/lib/helper.ts helperOnly",
            "web/src/util.ts utilOnly",
        ]
    );
}

#[test]
fn entrypoint_files_follow_the_shared_inventory_and_project_type() {
    let root = Path::new("/repo");
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some("apps/web".to_string()),
            ..Default::default()
        },
    );
    config.projects.insert(
        "other".to_string(),
        Project {
            type_: Some(ProjectType::Library),
            root: Some("other".to_string()),
            ..Default::default()
        },
    );
    let roots = project_roots(root, &config);
    assert_eq!(roots, vec![PathBuf::from("/repo/apps/web")]);

    for rel in [
        "app/page.tsx",
        "app/layout.ts",
        "app/loading.jsx",
        "app/error.js",
        "app/global-error.mts",
        "app/not-found.cts",
        "app/template.mjs",
        "app/default.cjs",
        "app/route.ts",
        "app/manifest.ts",
        "app/robots.ts",
        "app/sitemap.ts",
        "app/icon.tsx",
        "app/apple-icon.tsx",
        "src/app/opengraph-image.tsx",
        "src/app/twitter-image.ts",
        "app/blog/[slug]/page.tsx",
        "app/(shop)/template.tsx",
        "middleware.ts",
        "src/middleware.js",
        "instrumentation.mts",
        "src/instrumentation.cts",
        "next.config.ts",
        "next.config.js",
        "next.config.mjs",
        "next.config.cjs",
        "next.config.mts",
        "next.config.cts",
    ] {
        assert!(
            is_entrypoint(&root.join("apps/web").join(rel), &roots),
            "{rel}"
        );
    }

    for rel in [
        "app/lib/helper.ts",
        "src/util.ts",
        "pages/page.tsx",
        "pages/app/page.tsx",
        "app/page.test.tsx",
        "app/page.png",
        "app/icon.png",
        "app/middleware.ts",
        "src/app/middleware.ts",
        "src/lib/middleware.ts",
        "src/next.config.ts",
        "next.config.tsx",
        "next.config.jsx",
        "lib/page.tsx",
        "src/pages/page.tsx",
    ] {
        assert!(
            !is_entrypoint(&root.join("apps/web").join(rel), &roots),
            "{rel}"
        );
    }
    assert!(!is_entrypoint(&root.join("other/app/page.tsx"), &roots));
    assert!(!is_entrypoint(&root.join("apps/web/app/page.tsx"), &[]));
    // An omitted root that inference cannot see contributes no entrypoints.
    let mut bare = NoMistakesConfig::default();
    bare.projects.insert(
        "web".to_string(),
        Project {
            type_: Some(ProjectType::Nextjs),
            ..Default::default()
        },
    );
    assert!(project_roots(Path::new("/nm-2797-missing-nextjs-root"), &bare).is_empty());
    // A `web/app` tree is not an entrypoint of a project rooted at the repo.
    assert!(!is_framework_entrypoint_file(
        Path::new("/repo/apps/web/app/page.tsx"),
        Path::new("/repo"),
    ));
    assert!(is_framework_entrypoint_file(
        Path::new("/repo/apps/web/app/page.tsx"),
        Path::new("/repo/apps/web"),
    ));
}
