use super::{sql_rel, AllowedMigration, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::collect_postgres_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_schema: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &Default::default(),
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    let mut seen_allowed_migrations = BTreeSet::new();
    for migration in &opts.allowed_migrations {
        if seen_allowed_migrations.insert(migration.clone()) {
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: migration.path.clone(),
            line: 1,
            message: format!(
                "duplicate postgres-no-add-column allowedMigrations entry: {}",
                allowed_migration_target(migration)
            ),
            import: None,
            target: Some(allowed_migration_target(migration)),
        });
    }
    let mut used_allowed_migrations = BTreeSet::new();
    for file in &facts.schema {
        let rel = sql_rel(root, &file.path);
        for column in &file.add_columns {
            let actual = AllowedMigration {
                path: rel.clone(),
                table: column.table_name.clone(),
                column: column.column_name.clone(),
                data_type: column.data_type.clone(),
                nullable: column.nullable,
                default: column.default.clone(),
            };
            if opts.allowed_migrations.contains(&actual) {
                used_allowed_migrations.insert(actual);
                continue;
            }
            let message = if opts.allowed_migrations.is_empty() {
                format!(
                    "{rel}:{}: migration files must not use ALTER TABLE ADD COLUMN; fold the column into the original CREATE TABLE definition",
                    column.line.max(1)
                )
            } else {
                format!(
                    "{rel}:{}: ALTER TABLE ADD COLUMN does not match an allowedMigrations entry; update the exact path, table, column, type, nullable, and default fields or fold the column into the original CREATE TABLE definition",
                    column.line.max(1)
                )
            };
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: column.line.max(1),
                message,
                import: None,
                target: Some(format!("{}.{}", column.table_name, column.column_name)),
            });
        }
    }
    findings.extend(
        opts.allowed_migrations
            .iter()
            .filter(|migration| !used_allowed_migrations.contains(*migration))
            .map(stale_allowed_migration),
    );
    Ok(findings)
}

fn stale_allowed_migration(migration: &AllowedMigration) -> RuleFinding {
    let target = allowed_migration_target(migration);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: migration.path.clone(),
        line: 1,
        message: format!("stale postgres-no-add-column allowedMigrations entry: {target}"),
        import: None,
        target: Some(target),
    }
}

fn allowed_migration_target(migration: &AllowedMigration) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        migration.path,
        migration.table,
        migration.column,
        migration.data_type,
        migration.nullable,
        migration.default.as_deref().unwrap_or("<none>")
    )
}
