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
