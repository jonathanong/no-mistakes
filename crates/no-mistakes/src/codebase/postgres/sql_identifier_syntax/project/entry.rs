use super::*;
pub(super) type Declared = std::collections::BTreeMap<
    EmbeddedSqlOptions,
    Vec<crate::config::v2::schema::EffectTargetConfig>,
>;
#[cfg(test)]
pub(crate) fn collect(
    program: &Program<'_>,
    ts: &TsFileFacts,
    source: &str,
    profiles: &[EmbeddedSqlOptions],
) -> FileFacts {
    collect_with_scopes(
        program,
        ts,
        source,
        profiles,
        &Default::default(),
        &Default::default(),
        &mut Default::default(),
    )
}

pub(crate) fn collect_with_scopes(
    program: &Program<'_>,
    ts: &TsFileFacts,
    source: &str,
    profiles: &[EmbeddedSqlOptions],
    declared_types: &Declared,
    declared_factories: &Declared,
    scopes: &mut crate::codebase::postgres::embedded::scoped_bindings::PreparedScopes,
) -> FileFacts {
    collect_prepared(
        program,
        ts,
        source,
        profiles,
        declared_types,
        declared_factories,
        scopes,
    )
}
