use regex::Regex;
use std::sync::OnceLock;

// Static shapes from docs/feature-parity.md. Runtime topics and regex subscriptions stay unmatched.
pub(super) fn extra_produces(source: &str) -> Vec<String> {
    let mut values = Vec::new();
    push_group(&mut values, source, go_record_re());
    push_group(&mut values, source, rust_record_re());
    push_group(&mut values, source, named_produce_re());
    push_group(&mut values, source, php_topic_re());
    push_group(&mut values, source, java_record_re());
    push_group(&mut values, source, elixir_produce_re());
    values
}

pub(super) fn extra_consumes(source: &str) -> Vec<String> {
    let mut values = Vec::new();
    push_quoted_block(&mut values, source, go_subscribe_re());
    push_group(&mut values, source, ruby_subscribe_re());
    push_quoted_block(&mut values, source, java_subscribe_re());
    push_group(&mut values, source, elixir_subscribe_re());
    values
}

fn push_group(values: &mut Vec<String>, source: &str, re: &Regex) {
    for cap in re.captures_iter(source) {
        let Some(topic) = cap.get(1) else {
            continue;
        };
        if let Some(literal) = closed_literal(source, topic.start(), topic.end(), topic.as_str()) {
            values.push(literal);
        }
    }
}

fn push_quoted_block(values: &mut Vec<String>, source: &str, re: &Regex) {
    for cap in re.captures_iter(source) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        for element in block.split(',') {
            if let Some(literal) = bare_subscribe_literal(element.trim()) {
                values.push(literal);
            }
        }
    }
}

fn closed_literal(source: &str, start: usize, end: usize, text: &str) -> Option<String> {
    let quote = source[..start].chars().next_back()?;
    if (quote != '"' && quote != '\'') || concatenated(source, end) {
        return None;
    }
    if quote == '"' && text.contains("#{") {
        return None;
    }
    let closing = source[end..].chars().next()?;
    (closing == quote).then(|| text.to_string())
}

fn concatenated(source: &str, end: usize) -> bool {
    let rest = source[end..].trim_start();
    let Some(quote) = rest.chars().next() else {
        return false;
    };
    rest[quote.len_utf8()..].trim_start().starts_with('+')
}

fn bare_subscribe_literal(element: &str) -> Option<String> {
    let quote = element.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = element.strip_prefix(quote)?.strip_suffix(quote)?;
    if inner.is_empty() || inner.contains(quote) || (quote == '"' && inner.contains("#{")) {
        return None;
    }
    if inner.starts_with('^') || inner.contains('*') {
        return None;
    }
    Some(inner.to_string())
}

fn go_record_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:Writer|ProducerMessage)\s*\{[^}]*\bTopic\s*:\s*["']([^"']+)["']"#)
            .expect("go record")
    })
}

fn rust_record_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:FutureRecord|BaseRecord)::to\(\s*["']([^"']+)["']"#).expect("rust record")
    })
}

fn named_produce_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:\.|#)produce\(\s*[^)\n]*\btopic\s*:\s*["']([^"']+)["']"#)
            .expect("named produce")
    })
}

fn php_topic_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"newTopic\(\s*["']([^"']+)["']"#).expect("php topic"))
}

fn java_record_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\bProducerRecord\s*(?:<[^>\n]*>)?\(\s*["']([^"']+)["']"#)
            .expect("java record")
    })
}

fn elixir_produce_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#":brod\.produce_sync\(\s*[^,\n]+,\s*["']([^"']+)["']"#)
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
    RE.get_or_init(|| Regex::new(r#"subscribe\(\s*["']([^"']+)["']"#).expect("ruby subscribe"))
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
        Regex::new(r#":brod\.subscribe\(\s*[^,\n]+,\s*[^,\n]+,\s*["']([^"']+)["']"#)
            .expect("elixir subscribe")
    })
}
