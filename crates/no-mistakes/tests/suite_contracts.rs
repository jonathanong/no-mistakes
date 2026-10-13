//! Contracts integration tests, linked in one harness.

#[path = "ci_complexity_gate.rs"]
mod ci_complexity_gate;
#[path = "docs_coverage.rs"]
mod docs_coverage;
#[path = "support/integration_suites.rs"]
mod integration_suites;
#[path = "mermaid_validation.rs"]
mod mermaid_validation;
#[path = "release_validate_cache.rs"]
mod release_validate_cache;
#[path = "rule_finding_api.rs"]
mod rule_finding_api;
#[path = "shared_facts.rs"]
mod shared_facts;
