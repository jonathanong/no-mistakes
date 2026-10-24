use super::*;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::{EffectiveRuleAnalysis, Effort, Unanalyzable};

mod integrity;
mod routing;
mod work;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/declared-payload-compatibility/effort-unit"),
    )
}

fn contract(producer: &str, consumer: &str) -> DeclaredPayloadContract {
    DeclaredPayloadContract {
        name: format!("{producer}-to-{consumer}"),
        producer: reference(producer),
        consumer: reference(consumer),
    }
}

fn reference(name: &str) -> DeclaredPayloadSchema {
    DeclaredPayloadSchema {
        file: "schemas.json".into(),
        pointer: format!("/{name}"),
    }
}

fn rule(
    effort: Effort,
    unanalyzable: Unanalyzable,
    contracts: Vec<DeclaredPayloadContract>,
) -> RuleDef {
    RuleDef {
        rule: RULE_ID.into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::to_value(DeclaredPayloadCompatibilityOptions { contracts }).unwrap(),
        resolved_analysis: Some(EffectiveRuleAnalysis {
            effort,
            unanalyzable,
        }),
        ..RuleDef::default()
    }
}

fn run(rules: Vec<RuleDef>) -> AnalysisDiagnostics {
    let files = [root().join("schemas.json")];
    let sources = super::super::source_store_for_files(&files);
    diagnostics_with_files_and_sources(
        &root(),
        &NoMistakesConfig {
            rules,
            ..NoMistakesConfig::default()
        },
        &files,
        &sources,
        true,
    )
    .unwrap()
}
