use super::nextjs::project_roots;
use super::tests::fixture;
use crate::codebase::rules::run_check;
use crate::codebase::unique_exports::{framework_consumed_export, is_framework_entrypoint_file};
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
    // Framework-consumed names stay quiet. Other exports on those files are
    // reported. `fromPage` is imported by `app/page.tsx`, a production consumer.
    assert_eq!(
        actual,
        [
            "other/app/page.tsx otherPage",
            "web/app/apple-icon.tsx appleIconMarker",
            "web/app/blog/[slug]/page.tsx nestedMarker",
            "web/app/default.tsx defaultMarker",
            "web/app/error.tsx errorMarker",
            "web/app/global-error.tsx globalErrorMarker",
            "web/app/icon.tsx iconMarker",
            "web/app/layout.tsx layoutMarker",
            "web/app/lib/helper.ts helperOnly",
            "web/app/loading.tsx loadingMarker",
            "web/app/manifest.ts manifestMarker",
            "web/app/not-found.tsx notFoundMarker",
            "web/app/page.tsx pageMarker",
            "web/app/robots.ts robotsMarker",
            "web/app/route.ts routeMarker",
            "web/app/sitemap.ts sitemapMarker",
            "web/app/template.tsx templateMarker",
            "web/pages/_hidden.ts hiddenMarker",
            "web/pages/index.tsx pagesMarker",
            "web/proxy.ts proxyMarker",
            "web/src/app/opengraph-image.tsx opengraphMarker",
            "web/src/app/twitter-image.tsx twitterMarker",
            "web/src/instrumentation-client.ts clientMarker",
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
        "pages/index.tsx",
        "pages/page.tsx",
        "pages/app/page.tsx",
        "pages/blog/[slug].tsx",
        "pages/api/health.ts",
        "pages/_app.tsx",
        "pages/_document.tsx",
        "pages/_error.tsx",
        "src/pages/page.tsx",
        "src/pages/index.ts",
        "proxy.ts",
        "src/proxy.ts",
        "instrumentation-client.ts",
        "src/instrumentation-client.ts",
    ] {
        assert!(
            is_framework_entrypoint_file(&root.join("apps/web").join(rel), &roots[0]),
            "{rel}"
        );
    }

    for rel in [
        "app/lib/helper.ts",
        "src/util.ts",
        "pages/_hidden.ts",
        "pages/_utils.ts",
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
        "app/proxy.ts",
        "src/app/proxy.ts",
        "src/lib/instrumentation-client.ts",
        "src/proxy/proxy.ts",
    ] {
        assert!(
            !is_framework_entrypoint_file(&root.join("apps/web").join(rel), &roots[0]),
            "{rel}"
        );
    }
    assert!(!is_framework_entrypoint_file(
        &root.join("other/app/page.tsx"),
        &roots[0],
    ));
    // Outside the project root, and a non-code file under it, consume nothing.
    assert!(!framework_consumed_export(
        &root.join("other/app/page.tsx"),
        &roots[0],
        "default",
    ));
    assert!(!framework_consumed_export(
        &roots[0].join("app/page.png"),
        &roots[0],
        "default",
    ));
    assert!(!super::nextjs::consumed_export(
        &root.join("apps/web/app/page.tsx"),
        &[],
        "default",
    ));
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
