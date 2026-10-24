use super::*;

#[test]
fn fact_plan_keeps_boundary_only_rules_to_boundary_facts() {
    let boundary_only = fact_plan(EnabledChecks {
        boundary_rules: true,
        ..Default::default()
    });

    assert!(boundary_only.server_route_client_boundary);
    assert!(!boundary_only.source);
    assert!(!boundary_only.imports);
    assert!(!boundary_only.dynamic_imports);

    let dynamic_import_rule = fact_plan(EnabledChecks {
        dynamic_import_rules: true,
        ..Default::default()
    });

    assert!(dynamic_import_rule.source);
    assert!(dynamic_import_rule.imports);
    assert!(dynamic_import_rule.dynamic_imports);
    assert_eq!(
        dynamic_import_rule.graph,
        no_mistakes::codebase::ts_source::facts::TsFactPlan::imports()
    );

    let nextjs_caching = fact_plan(EnabledChecks {
        nextjs_caching: true,
        ..Default::default()
    });

    assert!(nextjs_caching.source);
    assert!(nextjs_caching.nextjs_caching);

    let nextjs_api_routes = fact_plan(EnabledChecks {
        nextjs_api_routes: true,
        ..Default::default()
    });

    assert!(nextjs_api_routes.raw_source);
    assert!(!nextjs_api_routes.source);
    assert!(!nextjs_api_routes.nextjs_caching);

    let lock_ordering = fact_plan(EnabledChecks {
        embedded_sql: true,
        ..Default::default()
    });
    assert!(lock_ordering.embedded_sql);
    assert!(!lock_ordering.postgres_schema);
}

#[test]
fn complete_domain_checks_surfaces_each_domain_error() {
    assert_domain_error(err_react(), "react");
    assert_domain_error(err_queues(), "queues");
    assert_domain_error(err_rules(), "rules");
    assert_domain_error(err_integration(), "integration");
    assert_domain_error(err_codebase(), "codebase");
    assert_domain_error(err_filesystem_rules(), "filesystem_rules");
}

fn assert_domain_error(results: DomainResults, expected: &str) {
    let err = complete_domain_checks(results)
        .err()
        .expect("expected domain check error");
    assert_eq!(err.to_string(), expected);
}
