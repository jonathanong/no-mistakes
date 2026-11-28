use super::tests_kafka::welcome;
use crate::codebase::lang_frontends::kafka::{extract_kafka_topics_for, KafkaFile};
use std::path::Path;

#[test]
fn kafka_operation_names_need_a_leading_identifier_boundary() {
    let rb = KafkaFile::from_path(Path::new("w.rb"));
    let php = KafkaFile::from_path(Path::new("w.php"));
    let rs = KafkaFile::from_path(Path::new("w.rs"));
    assert!(
        extract_kafka_topics_for("consumer.unsubscribe(\"mail.welcome\")", rb)
            .1
            .is_empty()
    );
    assert!(
        extract_kafka_topics_for("renewTopic(\"mail.welcome\")", php)
            .0
            .is_empty()
    );
    assert!(
        extract_kafka_topics_for("MyFutureRecord::to(\"mail.welcome\")", rs)
            .0
            .is_empty()
    );
}

#[test]
fn shared_send_path_rejects_interpolated_topics() {
    let rb = KafkaFile::from_path(Path::new("w.rb"));
    let kt = KafkaFile::from_path(Path::new("w.kt"));
    let dart = KafkaFile::from_path(Path::new("w.dart"));
    assert!(extract_kafka_topics_for("bus.send(\"mail.#{env}\")", rb)
        .0
        .is_empty());
    assert!(extract_kafka_topics_for("bus.send(\"mail.$env\")", kt)
        .0
        .is_empty());
    assert!(extract_kafka_topics_for("bus.send('mail.$env')", dart)
        .0
        .is_empty());
}

#[test]
fn ruby_block_comment_markers_need_a_trailing_boundary() {
    let source = "=begin\n=ending\nproducer.produce(payload, topic: \"ghost\")\n=end\n";
    let file = KafkaFile::from_path(Path::new("send.rb"));
    assert!(extract_kafka_topics_for(source, file).0.is_empty());
}

#[test]
fn ruby_block_comments_do_not_yield_topics() {
    let source = "=begin\nproducer.produce(payload, topic: \"ghost\")\nconsumer.subscribe(\"ghost.in\")\n=end\nproducer.produce(payload, topic: \"mail.welcome\")\n";
    let (produces, consumes) =
        extract_kafka_topics_for(source, KafkaFile::from_path(Path::new("send.rb")));
    assert_eq!(produces, welcome());
    assert!(consumes.is_empty(), "{consumes:?}");
}
