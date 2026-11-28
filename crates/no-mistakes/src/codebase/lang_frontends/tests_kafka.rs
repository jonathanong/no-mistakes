use crate::codebase::lang_frontends::kafka::{extract_kafka_topics_for, KafkaFile};
use std::path::{Path, PathBuf};

pub(super) fn extract_kafka_topics(source: &str) -> (Vec<String>, Vec<String>) {
    // Callers without a path keep the historical Ruby-subscribe scan.
    extract_kafka_topics_for(source, KafkaFile::new(true, false))
}

fn source(root: &str, rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends")
        .join(root)
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn topics(root: &str, rel: &str) -> (Vec<String>, Vec<String>) {
    extract_kafka_topics_for(&source(root, rel), KafkaFile::from_path(Path::new(rel)))
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

#[test]
fn dynamic_kafka_shapes_stay_unmatched() {
    let source = r#"
        fn publish<'a>(topic: &str) {
            // FutureRecord::to("ghost")
            let _ = FutureRecord::to("mail.welcome");
            let _ = FutureRecord::to("mail." + topic);
        }
        consumer.subscribe('mail.#{tenant}')
        consumer.subscribe("mail.#{tenant}")
        consumer.SubscribeTopics([]string{"^mail.*"}, nil)
        consumer.subscribe(List.of(resolveTopic("tenant")))
    "#;
    let (produces, consumes) = extract_kafka_topics(source);
    assert_eq!(produces, welcome());
    assert_eq!(consumes, vec!["mail.#{tenant}".to_string()]);
}

#[test]
fn kafka_file_kind_rejects_templates_concatenation_and_foreign_subscribe() {
    assert_eq!(
        extract_kafka_topics("$producer->newTopic('mail.$tenant');").0,
        vec!["mail.$tenant".to_string()]
    );
    assert!(
        extract_kafka_topics("$producer->newTopic(\"mail.$tenant\");")
            .0
            .is_empty()
    );
    let kotlin = "ProducerRecord(\"mail.$tenant\", null)";
    assert!(
        extract_kafka_topics_for(kotlin, KafkaFile::from_path(Path::new("publish.kt")))
            .0
            .is_empty()
    );
    assert_eq!(
        extract_kafka_topics_for(kotlin, KafkaFile::from_path(Path::new("Publish.java"))).0,
        vec!["mail.$tenant".to_string()]
    );
    let ruby_shape = "client.subscribe(\"mail.welcome\")";
    assert!(
        extract_kafka_topics_for(ruby_shape, KafkaFile::from_path(Path::new("worker.ts")))
            .1
            .is_empty()
    );
    assert_eq!(
        extract_kafka_topics_for(ruby_shape, KafkaFile::from_path(Path::new("worker.rb"))).1,
        welcome()
    );
    let rust_array =
        "consumer.subscribe(&[topic_for(\"tenant\")]);\nconsumer.subscribe(&[\"mail.welcome\"]);";
    assert_eq!(extract_kafka_topics(rust_array).1, welcome());
    assert!(extract_kafka_topics("FutureRecord::to(\"mail.\nwelcome\")")
        .0
        .is_empty());
    let concatenated = "producer.send({ topic: \"mail.\" + tenant });\nconsumer.subscribe({ topic: \"mail.\" + name });\nproducer.send({ topic: \"mail.welcome\" });";
    let (produces, consumes) = extract_kafka_topics(concatenated);
    assert_eq!(produces, welcome());
    assert!(consumes.is_empty());
    assert!(extract_kafka_topics_for(
        "$consumer->subscribe([\"mail.$tenant\"]);",
        KafkaFile::from_path(Path::new("listen.php")),
    )
    .1
    .is_empty());
    assert!(extract_kafka_topics_for(
        "consumer.subscribe(listOf(\"mail.$tenant\"))",
        KafkaFile::from_path(Path::new("listen.kt")),
    )
    .1
    .is_empty());
}
