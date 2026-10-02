use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::collect_prepared_schema_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    _sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_prepared_schema_facts(root, files, &opts.schema, facts)
        .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    for file in &facts {
        let rel = sql_rel(root, &file.path);
        for fk in &file.foreign_keys {
            if !missing_explicit_on_delete(fk.delete_action.as_deref()) {
                continue;
            }
            let column = fk
                .column_names
                .first()
                .map(String::as_str)
                .unwrap_or("foreign key");
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: fk.line.max(1),
                message: format!(
                    "{rel}:{}: foreign keys must declare an explicit ON DELETE action",
                    fk.line.max(1)
                ),
                import: None,
                target: Some(format!("{}.{}", fk.table_name, column)),
            });
        }
    }
    Ok(findings)
}

fn missing_explicit_on_delete(action: Option<&str>) -> bool {
    matches!(action, None | Some("NO ACTION"))
}
