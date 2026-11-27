mod matcher;
mod template;

use regex::Regex;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone)]
pub struct SelectorRegexes {
    pub(crate) app_attributes: Vec<String>,
    pub(crate) component_attributes: std::collections::BTreeMap<String, String>,
    pub(crate) playwright_attributes: Vec<AttributeRegex>,
    pub(crate) html_ids: bool,
}

impl SelectorRegexes {
    pub(crate) fn includes_html_ids(&self) -> bool {
        self.html_ids
    }
}

#[derive(Clone)]
pub(crate) struct AttributeRegex {
    pub(crate) attribute: String,
    pub(crate) regex: Regex,
}

#[derive(Clone, Debug)]
pub struct AppSelector {
    pub file: PathBuf,
    pub attribute: String,
    pub value: AppSelectorValue,
    pub line: usize,
    /// True when the exact value was recovered from an identifier, helper,
    /// ternary, object map, or imported binding rather than a literal.
    pub recovered: bool,
}

impl PartialEq for AppSelector {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file && self.attribute == other.attribute && self.value == other.value
    }
}

impl Eq for AppSelector {}

impl PartialOrd for AppSelector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AppSelector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file
            .cmp(&other.file)
            .then_with(|| self.attribute.cmp(&other.attribute))
            .then_with(|| self.value.cmp(&other.value))
    }
}

impl AppSelector {
    pub(crate) fn is_literal_exact(&self) -> bool {
        matches!(self.value, AppSelectorValue::Exact(_)) && !self.recovered
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AppSelectorValue {
    Exact(String),
    Template(TemplatePattern),
    Unsupported(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TemplatePattern {
    pub(crate) raw: String,
    pub(crate) parts: Vec<String>,
    pub(crate) starts_static: bool,
    pub(crate) ends_static: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlaywrightSelector {
    pub attribute: String,
    pub selector: String,
    pub(crate) matcher: SelectorMatcher,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlaywrightHelperReference {
    pub value: String,
    pub call: String,
}

impl PlaywrightSelector {
    pub fn for_test(attribute: &str, selector: &str, matcher: SelectorMatcher) -> Self {
        Self {
            attribute: attribute.into(),
            selector: selector.into(),
            matcher,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelectorMatcher {
    Exact(String),
    Prefix(String),
    Suffix(String),
    Contains(String),
    Regex {
        pattern: String,
        compiled: Option<Regex>,
    },
}

impl PartialEq for SelectorMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_key() == other.cmp_key()
    }
}

impl Eq for SelectorMatcher {}

impl PartialOrd for SelectorMatcher {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SelectorMatcher {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_key().cmp(&other.cmp_key())
    }
}

impl SelectorMatcher {
    pub fn cmp_key(&self) -> (u8, &str) {
        match self {
            Self::Exact(value) => (0, value),
            Self::Prefix(value) => (1, value),
            Self::Suffix(value) => (2, value),
            Self::Contains(value) => (3, value),
            Self::Regex { pattern, .. } => (4, pattern),
        }
    }
}

pub(crate) const HTML_ID_ATTRIBUTE: &str = "id";

pub fn compile_selector_regexes(
    attributes: &[String],
    component_attributes: &BTreeMap<String, String>,
) -> SelectorRegexes {
    compile_selector_regexes_with_html_ids(attributes, component_attributes, false)
}

pub fn compile_selector_regexes_with_html_ids(
    attributes: &[String],
    component_attributes: &BTreeMap<String, String>,
    html_ids: bool,
) -> SelectorRegexes {
    let mut playwright_attributes: Vec<_> = attributes
        .iter()
        .chain(component_attributes.values())
        .cloned()
        .collect();
    if html_ids {
        playwright_attributes.push(HTML_ID_ATTRIBUTE.to_string());
    }
    playwright_attributes.sort();
    playwright_attributes.dedup();

    SelectorRegexes {
        app_attributes: attributes.to_vec(),
        component_attributes: component_attributes.clone(),
        playwright_attributes: playwright_attributes
            .iter()
            .map(|attribute| AttributeRegex {
                attribute: attribute.clone(),
                regex: playwright_selector_regex(attribute),
            })
            .collect(),
        html_ids,
    }
}

fn playwright_selector_regex(attribute: &str) -> Regex {
    let pattern = format!(
        r#"\[\s*{}\s*(=|\^=|\$=|\*=)\s*(?:"([^"]+)"|'([^']+)')\s*\]"#,
        regex::escape(attribute)
    );
    Regex::new(&pattern).expect("valid Playwright selector regex")
}
