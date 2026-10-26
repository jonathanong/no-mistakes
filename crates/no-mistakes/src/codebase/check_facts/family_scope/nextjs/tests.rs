macro_rules! record_membership_normalization {
    () => {
        tests::record()
    };
}

thread_local! {
    static MEMBERSHIP_NORMALIZATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record() {
    MEMBERSHIP_NORMALIZATIONS.with(|count| count.set(count.get() + 1));
}

pub(crate) fn take_membership_normalizations() -> usize {
    MEMBERSHIP_NORMALIZATIONS.with(|count| count.replace(0))
}

#[test]
fn api_route_source_scope_uses_union_roots_before_rule_include() {
    use super::{configure_nextjs_api_route_source_scope, FamilyPathScope};
    use crate::codebase::check_facts::CheckFactPlan;
    use crate::config::v2::schema::{Project, ProjectType, RuleDef};
    use std::path::PathBuf;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-api-routes/fixture");
    // Standalone callers can retain `..` in root while the visible inventory is normalized.
    let route =
        crate::codebase::ts_resolver::normalize_path(&root.join("web/app/api/users/route.ts"));
    let other = crate::codebase::ts_resolver::normalize_path(&root.join("web/app/page.tsx"));
    let visible = [route.clone(), other.clone()];
    let inferred = crate::codebase::config::InferredRoots::default();
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "web".into(),
        Project {
            type_: Some(ProjectType::Nextjs),
            root: Some("web".into()),
            ..Default::default()
        },
    );
    let mut plan = CheckFactPlan {
        raw_source: true,
        ..Default::default()
    };
    configure_nextjs_api_route_source_scope(&root, &visible, &config, &inferred, &mut plan);
    assert_eq!(plan.raw_source_paths, FamilyPathScope::All);

    config.rules.push(RuleDef {
        rule: crate::codebase::rules::NEXTJS_NO_API_ROUTES.into(),
        projects: vec!["web".into()],
        include: vec!["web/app/elsewhere/**".into()],
        ..Default::default()
    });
    configure_nextjs_api_route_source_scope(&root, &visible, &config, &inferred, &mut plan);
    assert!(super::super::applies(&plan.raw_source_paths, &route));
    assert!(!super::super::applies(&plan.raw_source_paths, &other));

    configure_nextjs_api_route_source_scope(&root, &[route], &config, &inferred, &mut plan);
    assert_eq!(plan.raw_source_paths, FamilyPathScope::All);
    plan.raw_source = false;
    configure_nextjs_api_route_source_scope(&root, &visible, &config, &inferred, &mut plan);
    assert_eq!(plan.raw_source_paths, FamilyPathScope::All);
}
