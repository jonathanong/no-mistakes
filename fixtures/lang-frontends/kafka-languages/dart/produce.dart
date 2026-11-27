Future<void> publish(Producer producer, String topicName) async {
  await producer.produce(topic: "mail.welcome", messages: const []);
  await producer.produce(topic: topicName, messages: const []);
}
