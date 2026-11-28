use super::{parse_options_value, to_napi_error, to_pretty_json, Path};
use crate::options::CiTopologyOptions;
use no_mistakes::codebase::workflow_topology::render_mermaid::render_workflow_topology_mermaid;

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum Format {
    #[default]
    Json,
    Mermaid,
}

pub(crate) fn ci_topology_json_impl(mut input: serde_json::Value) -> napi::Result<String> {
    let format: Format = input
        .as_object_mut()
        .and_then(|fields| fields.remove("format"))
        .map(parse_options_value)
        .transpose()?
        .unwrap_or_default();
    let options = parse_options_value::<CiTopologyOptions>(input)?;
    let root = options.root.as_deref().unwrap_or(".");
    let report = no_mistakes::ci::topology_report(
        Path::new(root),
        options.config.as_deref().map(Path::new),
        &options.workflows,
    )
    .map_err(to_napi_error)?;
    match format {
        Format::Json => to_pretty_json(&report),
        Format::Mermaid if report.diagnostics.is_empty() => {
            to_pretty_json(&render_workflow_topology_mermaid(&report))
        }
        Format::Mermaid => {
            let messages = report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Err(napi::Error::from_reason(messages))
        }
    }
}

#[cfg(test)]
mod tests;
