use super::options::parse_options_value;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValidateMermaidMarkdownOptions {
    content: String,
    file: Option<String>,
}

pub(crate) fn validate_mermaid_markdown_json_impl(
    options: serde_json::Value,
) -> napi::Result<String> {
    let options = parse_options_value::<ValidateMermaidMarkdownOptions>(options)?;
    let result = no_mistakes::mermaid_validation::validate_markdown(
        &options.content,
        options.file.as_deref(),
    );
    Ok(crate::cli::json_string(&result))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "mermaid/tests.rs"]
mod tests;
