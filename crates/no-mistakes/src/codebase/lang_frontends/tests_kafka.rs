use crate::codebase::lang_frontends::kafka::extract_kafka_topics;
use std::path::PathBuf;

fn source(root: &str, rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends")
        .join(root)
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn topics(root: &str, rel: &str) -> (Vec<String>, Vec<String>) {
    extract_kafka_topics(&source(root, rel))
}

fn welcome() -> Vec<String> {
    vec!["mail.welcome".to_string()]
}

#[test]
fn typescript_and_python_kafka_fixtures_keep_their_topics() {
    let (produces, _) = topics("kafka-topics", "producer.ts");
    let (_, consumes) = topics("kafka-topics", "consumer.ts");
    let (python, python_consumes) = topics("kafka-topics", "producer.py");
    assert_eq!(produces, welcome());
    assert_eq!(
        consumes,
        vec![
            "mail.welcome".to_string(),
            "orders".to_string(),
            "payments".to_string()
        ]
    );
    assert_eq!(python, welcome());
    assert!(python_consumes.is_empty());
}

fn assert_static_pair(produce: &str, consume: &str) {
    let (produces, produce_consumes) = topics("kafka-languages", produce);
    let (consume_produces, consumes) = topics("kafka-languages", consume);
    assert_eq!(produces, welcome(), "{produce}");
    assert!(produce_consumes.is_empty(), "{produce}");
    assert!(consume_produces.is_empty(), "{consume}");
    assert_eq!(consumes, welcome(), "{consume}");
}

#[test]
fn go_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("go/produce.go", "go/consume.go");
}

#[test]
fn rust_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("rust/produce.rs", "rust/consume.rs");
}

#[test]
fn ruby_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("ruby/produce.rb", "ruby/consume.rb");
}

#[test]
fn php_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("php/produce.php", "php/consume.php");
}

#[test]
fn java_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("java/produce.java", "java/consume.java");
}

#[test]
fn kotlin_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("kotlin/produce.kt", "kotlin/consume.kt");
}

#[test]
fn elixir_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("elixir/produce.ex", "elixir/consume.ex");
}

#[test]
fn dart_kafka_literals_skip_runtime_topics_and_regex_subscriptions() {
    assert_static_pair("dart/produce.dart", "dart/consume.dart");
}
