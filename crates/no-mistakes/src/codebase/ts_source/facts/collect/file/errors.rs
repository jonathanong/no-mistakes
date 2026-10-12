use super::TsFileFacts;

pub(crate) fn facts_from_collection_result(result: anyhow::Result<TsFileFacts>) -> TsFileFacts {
    match result {
        Ok(facts) => facts,
        // Indexable extensions are expected to have an OXC source type.
        // Reaching this branch means that allowlist and OXC support drifted.
        Err(error) => failure_facts(error.to_string()),
    }
}

pub(super) fn failure_facts(error: String) -> TsFileFacts {
    // Preserve request-level failures and the legacy parser diagnostic channel.
    TsFileFacts {
        operational_error: Some(error.clone()),
        parse_error: Some(error),
        ..TsFileFacts::default()
    }
}
