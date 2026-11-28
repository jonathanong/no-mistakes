use super::kafka_topic_text::{bare_subscribe_literal, closed_literal};
use regex::Regex;
use std::sync::OnceLock;

// Static shapes from docs/feature-parity.md. Runtime topics and regex subscriptions stay unmatched.
pub(super) fn extra_produces(source: &str, dollar_templates: bool) -> Vec<String> {
    let mut values = Vec::new();
    push_group(&mut values, source, go_record_re(), false);
    push_group(&mut values, source, rust_record_re(), false);
    push_group(&mut values, source, named_produce_re(), false);
    push_group(&mut values, source, php_topic_re(), true);
    push_group(&mut values, source, java_record_re(), dollar_templates);
    push_group(&mut values, source, elixir_produce_re(), false);
    values
}

pub(super) fn extra_consumes(
    source: &str,
    ruby_subscribe: bool,
    dollar_templates: bool,
) -> Vec<String> {
    let mut values = Vec::new();
    push_quoted_block(&mut values, source, go_subscribe_re(), false);
    if ruby_subscribe {
        push_group(&mut values, source, ruby_subscribe_re(), false);
    }
    push_quoted_block(&mut values, source, java_subscribe_re(), dollar_templates);
    push_group(&mut values, source, elixir_subscribe_re(), false);
    values
}

fn push_group(values: &mut Vec<String>, source: &str, re: &Regex, reject_dollar: bool) {
    for cap in re.captures_iter(source) {
        let Some(topic) = cap.get(1) else {
            continue;
        };
        if let Some(literal) = closed_literal(
            source,
            topic.start(),
            topic.end(),
            topic.as_str(),
            reject_dollar,
        ) {
            values.push(literal);
        }
    }
}

fn push_quoted_block(values: &mut Vec<String>, source: &str, re: &Regex, reject_dollar: bool) {
    for cap in re.captures_iter(source) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        for element in block.split(',') {
            if let Some(literal) = bare_subscribe_literal(element.trim(), reject_dollar) {
                values.push(literal);
            }
        }
    }
}

fn go_record_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:Writer|ProducerMessage)\s*\{[^}]*\bTopic\s*:\s*["']([^"'\r\n]+)["']"#)
            .expect("go record")
    })
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
        Regex::new(r#"(?:\.|#)produce\(\s*[^)\n]*\btopic\s*:\s*["']([^"'\r\n]+)["']"#)
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
