use crate::config::v2::schema::RuleDef;

/// A low-tier implementation declares every rule whose enforcement it assumes.
#[derive(Clone, Copy)]
pub struct RuleEffortSupport {
    pub rule: &'static str,
    pub partners: &'static [&'static str],
    /// Certifies semantic/profile/runner coverage before file coverage is tested.
    /// Unknown or unsupported selectors must return false.
    pub partner_scope: fn(&RuleDef, &RuleDef) -> bool,
}

// Each low implementation either names its enforced partners or reports
// unsupported syntax explicitly under the caller's unanalyzable policy.
pub(super) const SUPPORT: &[RuleEffortSupport] = &[
    RuleEffortSupport {
        rule: "postgres-explicit-columns",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "postgres-required-predicates",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "postgres-no-transformed-uuid-predicates",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "finite-set-consistency",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "declared-payload-compatibility",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "postgres-no-offset",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "github-actions-download-integrity",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "require-storybook-stories",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "nextjs-no-caching",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "github-actions-job-timeouts",
        partners: &["github-actions-step-timeouts"],
        partner_scope: crate::codebase::rules::github_actions_job_timeouts::partner_scope,
    },
    RuleEffortSupport {
        rule: "nextjs-redirect-destinations",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "postgres-no-mutable-sql-registry",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "postgres-no-unbounded-sql-identifier",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "postgres-require-query-annotation",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "test-no-unmocked-dynamic-imports",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "playwright-unique-test-ids",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "playwright-unique-html-ids",
        partners: &[],
        partner_scope: |_, _| false,
    },
    RuleEffortSupport {
        rule: "playwright-coverage",
        partners: &[],
        partner_scope: |_, _| false,
    },
];
