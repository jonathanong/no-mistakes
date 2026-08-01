use super::options::parse_options;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ValidateMermaidMarkdownOptions {
    content: String,
    file: Option<String>,
}

pub(crate) fn validate_mermaid_markdown_json_impl(options_json: String) -> napi::Result<String> {
    let options = parse_options::<ValidateMermaidMarkdownOptions>(&options_json)?;
    let result = no_mistakes::mermaid_validation::validate_markdown(
        &options.content,
        options.file.as_deref(),
    );
    serde_json::to_string(&result).map_err(|error| napi::Error::from_reason(error.to_string()))
}

#[cfg(test)]
#[path = "mermaid/tests.rs"]
mod tests;
