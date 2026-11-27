package mail;

class Publish {
  void send(Producer producer, String topic) {
    producer.send(new ProducerRecord<>("mail.welcome", null, "ok"));
    producer.send(new ProducerRecord<>(topic, null, "ok"));
  }
}
