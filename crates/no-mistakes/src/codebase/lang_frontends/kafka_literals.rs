use super::kafka::KafkaFile;
use super::kafka_go_record::go_outer_topic_spans;
use super::kafka_topic_text::{bare_subscribe_literal, closed_literal, DollarScope};
use regex::Regex;
use std::sync::OnceLock;

// Static shapes from docs/feature-parity.md. Runtime topics and regex subscriptions stay unmatched.
pub(super) fn extra_produces(source: &str, file: KafkaFile) -> Vec<String> {
    let mut values = Vec::new();
    if file.allows_go() {
        push_go_records(&mut values, source);
    }
    if file.allows_rust_records() {
        push_group(&mut values, source, rust_record_re(), DollarScope::None);
    }
    if file.allows_named_produce() {
        push_group(&mut values, source, named_produce_re(), file.dollar_scope());
    }
    if file.allows_php() {
        push_group(
            &mut values,
            source,
            php_topic_re(),
            DollarScope::DoubleQuote,
        );
    }
    if file.allows_java() {
        push_group(&mut values, source, java_record_re(), file.dollar_scope());
    }
    if file.allows_elixir() {
        push_group(&mut values, source, elixir_produce_re(), DollarScope::None);
    }
    values
}

pub(super) fn extra_consumes(source: &str, file: KafkaFile) -> Vec<String> {
    let mut values = Vec::new();
    if file.allows_go() {
        push_quoted_block(&mut values, source, go_subscribe_re(), DollarScope::None);
    }
    if file.allows_ruby() {
        push_group(&mut values, source, ruby_subscribe_re(), DollarScope::None);
    }
    if file.allows_java() {
        push_quoted_block(
            &mut values,
            source,
            java_subscribe_re(),
            file.dollar_scope(),
        );
    }
    if file.allows_elixir() {
        push_group(
            &mut values,
            source,
            elixir_subscribe_re(),
            DollarScope::None,
        );
    }
    values
}

fn push_go_records(values: &mut Vec<String>, source: &str) {
    for (start, end) in go_outer_topic_spans(source) {
        let text = &source[start..end];
        if let Some(literal) = closed_literal(source, start, end, text, DollarScope::None) {
            values.push(literal);
        }
    }
}

fn push_group(values: &mut Vec<String>, source: &str, re: &Regex, dollar_scope: DollarScope) {
    for cap in re.captures_iter(source) {
        let Some(topic) = cap.get(1) else {
            continue;
        };
        if let Some(literal) = closed_literal(
            source,
            topic.start(),
            topic.end(),
            topic.as_str(),
            dollar_scope,
        ) {
            values.push(literal);
        }
    }
}

fn push_quoted_block(
    values: &mut Vec<String>,
    source: &str,
    re: &Regex,
    dollar_scope: DollarScope,
) {
    for cap in re.captures_iter(source) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        for element in block.split(',') {
            if let Some(literal) = bare_subscribe_literal(element.trim(), dollar_scope) {
                values.push(literal);
            }
        }
    }
}

fn rust_record_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:FutureRecord|BaseRecord)::to\(\s*["']([^"'\r\n]+)["']"#)
            .expect("rust record")
    })
}

fn named_produce_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:\.|#)produce\(\s*[^)\n]*?\btopic\s*:\s*["']([^"'\r\n]+)["']"#)
            .expect("named produce")
    })
}

fn php_topic_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"newTopic\(\s*["']([^"'\r\n]+)["']"#).expect("php topic"))
}

fn java_record_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bProducerRecord\s*(?:<[^>\n]*>)?\(\s*["']([^"'\r\n]+)["']"#)
            .expect("java record")
    })
}

fn elixir_produce_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#":brod\.produce_sync\(\s*[^,\n]+,\s*["']([^"'\r\n]+)["']"#)
            .expect("elixir produce")
    })
}

fn go_subscribe_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"SubscribeTopics\(\s*\[\]string\s*\{([^}]*)\}"#).expect("go subscribe")
    })
}

fn ruby_subscribe_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"subscribe\(\s*["']([^"'\r\n]+)["']"#).expect("ruby subscribe"))
}

fn java_subscribe_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"subscribe\(\s*(?:List\.of|Arrays\.asList|Collections\.singletonList|listOf)\(([^)]*)\)"#,
        )
        .expect("java subscribe")
    })
}

fn elixir_subscribe_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#":brod\.subscribe\(\s*[^,\n]+,\s*[^,\n]+,\s*["']([^"'\r\n]+)["']"#)
            .expect("elixir subscribe")
    })
}
