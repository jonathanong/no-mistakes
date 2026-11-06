use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SchemaCatalog;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::path::Path;

/// Compile each application before path filtering, then read its prepared catalog.
pub(crate) fn run_catalog_rule<C>(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    rule_id: &str,
    compile: impl Fn(&RuleDef) -> Result<(String, C)>,
    scan: impl Fn(C, &SchemaCatalog, &str) -> Vec<RuleFinding>,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, rule_id, |rule| {
        let (path, compiled) = compile(rule)?;
        let Some(catalog) = super::selected_schema_catalog(root, config, rule, facts, &path)?
        else {
            return Ok(Vec::new());
        };
        Ok(scan(compiled, catalog, &path))
    })
}

pub(crate) fn run_audited_catalog_rule<C>(
    context: super::PreparedRuleContext<'_>,
    rule_id: &str,
    option: &str,
    compile: impl Fn(&RuleDef) -> Result<(String, C, Vec<String>)>,
    scan: impl Fn(&SchemaCatalog, &C, &str, &super::allow_audit::AllowUsage) -> Vec<RuleFinding>,
) -> Result<Vec<RuleFinding>> {
    run_catalog_rule(
        context.root,
        context.config,
        context.facts,
        rule_id,
        |rule| {
            let (path, compiled, selectors) = compile(rule)?;
            let audit = super::allow_audit::AllowAudit::new(
                context.root,
                context.config,
                rule,
                Some(context.sources),
                option,
                selectors.len(),
            );
            Ok((path, (compiled, audit, selectors)))
        },
        |(compiled, audit, selectors), catalog, path| {
            let mut findings = scan(catalog, &compiled, path, &audit.usage);
            findings.extend(audit.finish(
                |_| catalog.coverage() == crate::codebase::postgres::CatalogCoverage::Complete,
                |index| selectors[index].clone(),
            ));
            findings
        },
    )
}
