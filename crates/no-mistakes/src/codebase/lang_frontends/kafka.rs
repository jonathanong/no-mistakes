use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub(crate) fn extract_kafka_topics(source: &str) -> (Vec<String>, Vec<String>) {
    let source = super::strip::strip_comments_keep_strings(source);
    let produces = extract_named(&source, kafka_produce_re());
    let consumes = extract_kafka_consumes(&source);
    (produces, consumes)
}

pub(crate) fn topic_identity(cluster: Option<&str>, topic: &str) -> String {
    match cluster.filter(|value| !value.is_empty()) {
        Some(cluster) => format!("{cluster}:{topic}"),
        None => topic.to_string(),
    }
}

pub(crate) fn scan_file(path: &Path) -> Option<(Vec<String>, Vec<String>)> {
    let source = std::fs::read_to_string(path).ok()?;
    let text = super::strip::strip_comments_keep_strings(&source);
    Some(extract_kafka_topics(&text))
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| {
            cap.iter()
                .skip(1)
                .flatten()
                .map(|m| m.as_str().to_string())
                .next()
        })
        .collect();
    values.sort();
    values.dedup();
    values
}

fn extract_kafka_consumes(source: &str) -> Vec<String> {
    let mut values = extract_named(source, kafka_consume_re());
    for cap in kafka_consume_array_re().captures_iter(source) {
        values.extend(extract_named(
            cap.get(1).map(|m| m.as_str()).unwrap_or(""),
            kafka_quoted_re(),
        ));
    }
    values.sort();
    values.dedup();
    values
}

fn kafka_produce_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:\.|#)send\(\s*(?:\{[^}]*topic\s*:\s*["']([^"']+)["']|["']([^"']+)["'])"#)
            .expect("produce")
    })
}

fn kafka_consume_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"subscribe\(\s*\{[^}]*topic\s*:\s*["']([^"']+)["']"#).expect("consume")
    })
}

fn kafka_consume_array_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"subscribe\(\s*\[([^\]]*)\]"#).expect("consume array"))
}

fn kafka_quoted_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"["']([^"']+)["']"#).expect("quoted topic"))
}
