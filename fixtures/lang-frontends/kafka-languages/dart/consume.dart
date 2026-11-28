void listen(Consumer consumer, String topicName) {
  consumer.subscribe(["mail.welcome"]);
  consumer.subscribe([topicName]);
  consumer.subscribe(RegExp(r"mail.*"));
}
