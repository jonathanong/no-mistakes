use crate::config::v2::{schema::RuleDef, Effort, NoMistakesConfig};
use anyhow::Result;

const PARTNER: &str = "github-actions-step-timeouts";

/// Match the partner's supported options, refusing exceptions and unknown selectors.
pub(crate) fn partner_scope(_: &RuleDef, partner: &RuleDef) -> bool {
    partner.tests == Default::default()
        && (partner.options.is_null()
            || partner.options.as_mapping().is_some_and(|options| {
                options.keys().all(|key| {
                    matches!(
                        key.as_str(),
                        Some("include" | "maxMinutes" | "allow" | "effort" | "unanalyzable")
                    )
                })
            }))
        && partner
            .options
            .get("allow")
            .is_none_or(|allow| allow.as_sequence().is_some_and(Vec::is_empty))
        && partner
            .options
            .get("maxMinutes")
            .is_none_or(|cap| cap.as_u64().is_some())
        && partner.options.get("include").is_none_or(|include| {
            include
                .as_sequence()
                .is_some_and(|patterns| patterns.iter().all(|pattern| pattern.as_str().is_some()))
        })
}

pub(super) fn certified_cap(config: &NoMistakesConfig, rule: &RuleDef) -> Result<Option<u64>> {
    if config.effective_rule_analysis(rule)?.effort != Effort::Low {
        return Ok(None);
    }
    // Coverage may be supplied by several applications. The largest eligible
    // cap is safe for every covered file, including disjoint and overlapping scopes.
    Ok(config
        .rule_applications(PARTNER)
        .into_iter()
        .filter(|partner| partner_scope(rule, partner))
        .map(|partner| {
            partner
                .options
                .get("maxMinutes")
                .and_then(serde_yaml::Value::as_u64)
                .unwrap_or(360)
                .min(360)
        })
        .max())
}
