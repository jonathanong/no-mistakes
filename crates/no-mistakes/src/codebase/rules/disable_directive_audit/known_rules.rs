pub(super) fn known_rule(rule: &str) -> bool {
    use super::super::*;
    super::super::filesystem_dispatch::FILESYSTEM_RULE_IDS.contains(&rule)
        || [
            coverage_producer_assertion_owner::RULE_ID,
            production_exports_test_only::RULE_ID,
            FORBIDDEN_CALLS,
            FORBIDDEN_DEPENDENCIES,
            NEXTJS_NO_API_ROUTES,
            NEXTJS_NO_CACHING,
            POSTGRES_NO_MUTABLE_SQL_REGISTRY,
            POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
            QUERY_REACHED_PER_ITEM,
            REQUIRE_STORYBOOK_STORIES,
            REQUIRED_ENTRYPOINT_REACHABILITY,
            SERVER_ROUTE_CLIENT_BOUNDARY,
            TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
            UNCONSTRUCTED_ERROR_CLASS,
            "assert-no-fetch",
            "queues-check",
            "unique-exports",
            "playwright-coverage",
            "playwright-unique-test-ids",
            "playwright-unique-html-ids",
            "playwright-prefer-test-id-locators",
            "unused-disable-directive",
            "invalid-disable-directive",
        ]
        .contains(&rule)
}
