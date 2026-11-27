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
    values.extend(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    );
}

fn push_quoted_block(values: &mut Vec<String>, source: &str, re: &Regex) {
    for cap in re.captures_iter(source) {
        let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        push_group(values, block, quoted_re());
    }
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

fn quoted_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"["']([^"']+)["']"#).expect("quoted topic"))
}
