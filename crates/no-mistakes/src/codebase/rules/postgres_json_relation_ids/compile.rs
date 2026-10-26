use super::{
    CompiledOptions, EmbeddedSqlOptions, GlobMatcher, Options, PostgresSchemaOptions, RULE_ID,
};
use anyhow::{bail, Result};
use regex::Regex;

pub(super) fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    for (key, column) in &opts.key_column_map {
        if key.trim().is_empty() {
            bail!("{RULE_ID} option keyColumnMap: key must not be empty");
        }
        if column.trim().is_empty() {
            bail!("{RULE_ID} option keyColumnMap: column for {key} must not be empty");
        }
    }
    if opts.schema_catalog_path.trim().is_empty()
        && (!opts.key_column_map.is_empty() || opts.fingerprint_name_pattern.is_some())
    {
        bail!("{RULE_ID} option schemaCatalogPath: required when keyColumnMap or fingerprintNamePattern is set");
    }
    let id_key_pattern = pattern("idKeyPattern", opts.id_key_pattern.as_deref())?;
    let fingerprint_name_pattern = pattern(
        "fingerprintNamePattern",
        opts.fingerprint_name_pattern.as_deref(),
    )?;
    let mut allow = std::collections::BTreeSet::new();
    for entry in &opts.allow {
        if entry.reason.trim().is_empty() {
            bail!(
                "{RULE_ID} option allow: entry {} needs a reason",
                entry.object
            );
        }
        let object = entry
            .object
            .parse::<crate::codebase::postgres::CatalogObjectRef>();
        if !matches!(
            object,
            Ok(crate::codebase::postgres::CatalogObjectRef::Table(_)
                | crate::codebase::postgres::CatalogObjectRef::Column { .. })
        ) || entry.object.matches(':').count() != 1
        {
            bail!(
                "{RULE_ID} option allow: invalid object ref {}",
                entry.object
            );
        }
        if !allow.insert(entry.object.clone()) {
            bail!("{RULE_ID} option allow: duplicate entry {}", entry.object);
        }
    }
    GlobMatcher::new(&opts.sql_include, &format!("{RULE_ID} sqlInclude"))?;
    Ok(CompiledOptions {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        id_key_pattern,
        key_column_map: opts.key_column_map.clone(),
        fingerprint_name_pattern,
        allow,
        include: GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?,
        exclude: GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?,
        schema: PostgresSchemaOptions {
            sql_include: crate::codebase::postgres::effective_sql_include(
                &opts.sql_include,
                opts.sql_include_locked,
            ),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
    })
}

fn pattern(name: &str, raw: Option<&str>) -> Result<Option<Regex>> {
    raw.map(|raw| {
        if name == "idKeyPattern" && raw.is_empty() {
            bail!("{RULE_ID} option {name}: must not be empty");
        }
        Regex::new(raw).map_err(|error| anyhow::anyhow!("{RULE_ID} option {name}: {error}"))
    })
    .transpose()
}
