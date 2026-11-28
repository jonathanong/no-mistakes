use super::kafka_topic_text::DollarScope;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub(crate) enum KafkaFile {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "path-free unit tests construct this variant")
    )]
    Historical,
    Go,
    Rust,
    Ruby,
    Php,
    Java,
    Kotlin,
    Elixir,
    Dart,
    Shared,
}

impl KafkaFile {
    pub(crate) fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("go") => Self::Go,
            Some("rs") => Self::Rust,
            Some("rb") => Self::Ruby,
            Some("php") => Self::Php,
            Some("java") => Self::Java,
            Some("kt" | "kts") => Self::Kotlin,
            Some("ex" | "exs") => Self::Elixir,
            Some("dart") => Self::Dart,
            _ => Self::Shared,
        }
    }

    pub(super) fn dollar_scope(self) -> DollarScope {
        match self {
            Self::Php | Self::Kotlin => DollarScope::DoubleQuote,
            Self::Dart => DollarScope::AnyQuote,
            _ => DollarScope::None,
        }
    }

    fn rust_apostrophe(self) -> bool {
        matches!(self, Self::Historical | Self::Rust)
    }

    pub(super) fn allows_go(self) -> bool {
        matches!(self, Self::Historical | Self::Go)
    }

    pub(super) fn allows_rust_records(self) -> bool {
        matches!(self, Self::Historical | Self::Rust)
    }

    pub(super) fn allows_named_produce(self) -> bool {
        matches!(self, Self::Historical | Self::Ruby | Self::Dart)
    }

    pub(super) fn allows_php(self) -> bool {
        matches!(self, Self::Historical | Self::Php)
    }

    pub(super) fn allows_java(self) -> bool {
        matches!(self, Self::Historical | Self::Java | Self::Kotlin)
    }

    pub(super) fn allows_elixir(self) -> bool {
        matches!(self, Self::Historical | Self::Elixir)
    }

    pub(super) fn allows_ruby(self) -> bool {
        matches!(self, Self::Historical | Self::Ruby)
    }

    fn strip(self, source: &str) -> String {
        if self.rust_apostrophe() {
            super::strip::strip_comments_keep_strings_rust(source)
        } else {
            super::strip::strip_comments_keep_strings(source)
        }
    }
}

pub(crate) fn extract_kafka_topics_for(
    source: &str,
    file: KafkaFile,
) -> (Vec<String>, Vec<String>) {
    let source = file.strip(source);
    let mut produces = extract_named(&source, kafka_produce_re());
    produces.extend(super::kafka_literals::extra_produces(&source, file));
    let mut consumes = extract_kafka_consumes(&source, file.dollar_scope());
    consumes.extend(super::kafka_literals::extra_consumes(&source, file));
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
    let file = KafkaFile::from_path(path);
    let source = sources.read_path(path).ok()?;
    let text = file.strip(&source);
    Some(extract_kafka_topics_for(&text, file))
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

fn extract_kafka_consumes(source: &str, dollar_scope: DollarScope) -> Vec<String> {
    let mut values = extract_named(source, kafka_consume_re());
    for cap in kafka_consume_array_re().captures_iter(source) {
        let borrowed = cap.get(1).is_some();
        let block = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        for element in block.split(',') {
            let element = element.trim();
            let literal = if borrowed {
                super::kafka_topic_text::bare_subscribe_literal(element, dollar_scope)
            } else {
                super::kafka_topic_text::quoted_element(element, dollar_scope)
            };
            if let Some(literal) = literal {
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
    RE.get_or_init(|| Regex::new(r#"subscribe\(\s*(&)?\s*\[([^\]]*)\]"#).expect("consume array"))
}
