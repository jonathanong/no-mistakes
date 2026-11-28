use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub(crate) struct KafkaFile {
    ruby_subscribe: bool,
    dollar_templates: bool,
}

impl KafkaFile {
    pub(crate) fn new(ruby_subscribe: bool, dollar_templates: bool) -> Self {
        Self {
            ruby_subscribe,
            dollar_templates,
        }
    }

    pub(crate) fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rb") => Self::new(true, false),
            Some("kt" | "kts" | "php") => Self::new(false, true),
            _ => Self::new(false, false),
        }
    }
}

pub(crate) fn extract_kafka_topics_for(
    source: &str,
    file: KafkaFile,
) -> (Vec<String>, Vec<String>) {
    let source = super::strip::strip_comments_keep_strings(source);
    let mut produces = extract_named(&source, kafka_produce_re());
    produces.extend(super::kafka_literals::extra_produces(
        &source,
        file.dollar_templates,
    ));
    let mut consumes = extract_kafka_consumes(&source, file.dollar_templates);
    consumes.extend(super::kafka_literals::extra_consumes(
        &source,
        file.ruby_subscribe,
        file.dollar_templates,
    ));
    dedup(&mut produces);
    dedup(&mut consumes);
    (produces, consumes)
}

pub(crate) fn topic_identity(cluster: Option<&str>, topic: &str) -> String {
    match cluster.filter(|value| !value.is_empty()) {
        Some(cluster) => format!("{cluster}:{topic}"),
        None => topic.to_string(),
    }
}

pub(crate) fn scan_file(path: &Path, sources: &SourceStore) -> Option<(Vec<String>, Vec<String>)> {
    let source = sources.read_path(path).ok()?;
    let text = super::strip::strip_comments_keep_strings(&source);
    Some(extract_kafka_topics_for(&text, KafkaFile::from_path(path)))
}

fn dedup(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| {
            let matched = cap.iter().skip(1).flatten().next()?;
            super::kafka_topic_text::complete_topic(
                source,
                matched.start(),
                matched.end(),
                matched.as_str(),
            )
        })
        .collect();
    values.sort();
    values.dedup();
    values
}

fn extract_kafka_consumes(source: &str, dollar_templates: bool) -> Vec<String> {
    let mut values = extract_named(source, kafka_consume_re());
    for cap in kafka_consume_array_re().captures_iter(source) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        for element in block.split(',') {
            if let Some(literal) =
                super::kafka_topic_text::quoted_element(element.trim(), dollar_templates)
            {
                values.push(literal);
            }
        }
    }
    values.sort();
    values.dedup();
    values
}

fn kafka_produce_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?:\.|#)send\(\s*(?:\{[^}]*topic\s*:\s*["']([^"'\r\n]+)["']|["']([^"'\r\n]+)["'])"#,
        )
        .expect("produce")
    })
}

fn kafka_consume_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"subscribe\(\s*\{[^}]*topic\s*:\s*["']([^"'\r\n]+)["']"#).expect("consume")
    })
}

fn kafka_consume_array_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"subscribe\(\s*&?\s*\[([^\]]*)\]"#).expect("consume array"))
}
