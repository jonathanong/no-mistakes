package mail

fun publish(producer: Producer, topic: String) {
    producer.send(ProducerRecord("mail.welcome", null, "ok"))
    producer.send(ProducerRecord("mail.$tenant", null, "ok"))
    producer.send(ProducerRecord(topic, null, "ok"))
    producer.send(ProducerRecord("mail." + topic, null, "ok"))
}
