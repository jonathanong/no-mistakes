// Included into `options` via `include!`; shares that module's imports.

pub(crate) fn parse_queue_direction(
    value: Option<&str>,
) -> AnyhowResult<crate::queue::RelatedDirection> {
    match value.unwrap_or("both") {
        "deps" => Ok(crate::queue::RelatedDirection::Deps),
        "dependents" => Ok(crate::queue::RelatedDirection::Dependents),
        "both" => Ok(crate::queue::RelatedDirection::Both),
        value => bail!("unknown direction: {value}"),
    }
}

pub(crate) fn parse_server_direction(
    value: Option<&str>,
) -> AnyhowResult<crate::server_routes::RelatedDirection> {
    match value.unwrap_or("both") {
        "deps" => Ok(crate::server_routes::RelatedDirection::Deps),
        "dependents" => Ok(crate::server_routes::RelatedDirection::Dependents),
        "both" => Ok(crate::server_routes::RelatedDirection::Both),
        value => bail!("unknown direction: {value}"),
    }
}

pub(crate) fn to_napi_error(error: anyhow::Error) -> napi::Error {
    napi::Error::from_reason(format!("{error:#}"))
}
