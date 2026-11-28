use super::filesystem_dispatch::FILESYSTEM_RULE_IDS;
use super::*;
use crate::playwright::rules::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};

const OTHER_CONFIGURABLE_RULE_IDS: &[&str] = &[
    coverage_producer_assertion_owner::RULE_ID,
    production_exports_test_only::RULE_ID,
    FORBIDDEN_CALLS,
    FORBIDDEN_DEPENDENCIES,
    NEXTJS_NO_API_ROUTES,
    NEXTJS_NO_CACHING,
    POSTGRES_NO_MUTABLE_SQL_REGISTRY,
    POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
    POSTGRES_SQL_FRAGMENT_ESCAPE,
    QUERY_REACHED_PER_ITEM,
    NO_SINGLE_ENQUEUE_IN_LOOP,
    QUEUE_RATE_LIMIT_ERROR_DELAY,
    REQUIRE_STORYBOOK_STORIES,
    REQUIRED_ENTRYPOINT_REACHABILITY,
    NO_CROSS_PACKAGE_RELATIVE_IMPORTS,
    SERVER_ROUTE_CLIENT_BOUNDARY,
    TEST_NO_LIVE_REPOSITORY_SCAN,
    TEST_NO_UNMOCKED_DYNAMIC_IMPORTS,
    UNCONSTRUCTED_ERROR_CLASS,
    crate::codebase::unique_exports::RULE_ID,
    PLAYWRIGHT_COVERAGE,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
    PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
    "assert-no-fetch",
    "route-consistency",
    "queue-dashboard-reachability",
    "http-route-static-paths",
    "http-call-static-paths",
];

pub fn configurable_rule_ids() -> impl Iterator<Item = &'static str> {
    FILESYSTEM_RULE_IDS
        .iter()
        .chain(OTHER_CONFIGURABLE_RULE_IDS)
        .copied()
}
