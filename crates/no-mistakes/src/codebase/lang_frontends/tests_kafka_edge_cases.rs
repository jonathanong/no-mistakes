use super::tests_kafka::{extract_kafka_topics, welcome};
use crate::codebase::lang_frontends::kafka::{extract_kafka_topics_for, KafkaFile};
use std::path::Path;

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

#[test]
fn go_outer_topic_ignores_nested_strings_and_suffixed_types() {
    let before =
        "sarama.ProducerMessage{Metadata: metadata{Topic: \"audit\"}, Topic: \"mail.welcome\"}";
    let after =
        "sarama.ProducerMessage{Topic: \"mail.welcome\", Metadata: metadata{Topic: \"audit\"}}";
    let quoted_brace = "ProducerMessage{Note: \"\\\"}\", Topic: \"mail.welcome\"}";
    let raw_str = "s := `Writer{Topic: \"ghost\"}`\nw := kafka.Writer{Topic: \"mail.welcome\"}";
    let raw_url = "s := `https://host`\nw := kafka.Writer{Topic: \"mail.welcome\"}";
    let suffixed = "var w *kafka.Writer\nAuditWriter{Topic: \"audit\"}\nkafka.Writer{MyTopic: \"x\", TopicName: \"y\", Topic, Topic: topicVar, Topic: \"\", Topic: \"a\nb\", Note: 'x', Topic: \"mail.welcome\"}";
    let unclosed = "kafka.Writer{Topic: \"mail.welcome\"}\n_ = \"unterminated";
    for source in [
        before,
        after,
        quoted_brace,
        raw_str,
        raw_url,
        suffixed,
        unclosed,
    ] {
        assert_eq!(
            extract_kafka_topics_for(source, KafkaFile::from_path(Path::new("publish.go"))).0,
            welcome(),
            "{source}"
        );
    }
}

#[test]
fn named_produce_and_interpolation_stay_language_specific() {
    let source = "producer.produce({ topic: \"mail.welcome\" });";
    assert!(
        extract_kafka_topics_for(source, KafkaFile::from_path(Path::new("send.ts")))
            .0
            .is_empty()
    );
    assert_eq!(
        extract_kafka_topics_for(source, KafkaFile::from_path(Path::new("send.dart"))).0,
        welcome()
    );
    let ruby_nested =
        "producer.produce(payload, topic: \"mail.welcome\", headers: { topic: \"audit\" })";
    assert_eq!(
        extract_kafka_topics_for(ruby_nested, KafkaFile::from_path(Path::new("send.rb"))).0,
        welcome()
    );
    let dart_interp =
        "producer.produce(topic: 'mail.$tenant', messages: const []);\nproducer.produce(topic: 'mail.${tenant}', messages: const []);";
    assert!(
        extract_kafka_topics_for(dart_interp, KafkaFile::from_path(Path::new("send.dart")))
            .0
            .is_empty()
    );
    let dart_literal_dollar = "producer.produce(topic: 'mail.\\$1', messages: const []);\nproducer.produce(topic: 'mail.$1', messages: const []);";
    assert_eq!(
        extract_kafka_topics_for(
            dart_literal_dollar,
            KafkaFile::from_path(Path::new("send.dart")),
        )
        .0,
        vec!["mail.$1".to_string(), "mail.\\$1".to_string()]
    );
    let ruby_interp = "producer.produce(payload, topic: \"mail.#$tenant\")\nproducer.produce(payload, topic: \"mail.#@tenant\")";
    assert!(
        extract_kafka_topics_for(ruby_interp, KafkaFile::from_path(Path::new("send.rb")))
            .0
            .is_empty()
    );
}

#[test]
fn rust_arrays_lifetimes_and_php_multiline_quotes_keep_expected_topics() {
    let rust_regex = "consumer.subscribe(&[\"^mail.*\", \"mail.#{tenant}\", \"\", \"a\"\"b\", \"a\nb\", \"mail.welcome\"]);";
    assert_eq!(
        extract_kafka_topics_for(rust_regex, KafkaFile::from_path(Path::new("listen.rs"))).1,
        welcome()
    );
    let php_multiline =
        "$body = 'first\n\"';\n// producer.send(\"ghost\");\n$producer->newTopic(\"mail.welcome\");";
    assert_eq!(
        extract_kafka_topics_for(php_multiline, KafkaFile::from_path(Path::new("send.php"))).0,
        welcome()
    );
    let rust_lifetime = "fn publish<'a>(topic: &'a str) {\nlet q = '\\'';\n// FutureRecord::to(\"ghost\")\nlet _ = FutureRecord::to(\"mail.welcome\");\n}";
    assert_eq!(
        extract_kafka_topics_for(rust_lifetime, KafkaFile::from_path(Path::new("publish.rs"))).0,
        welcome()
    );
}
