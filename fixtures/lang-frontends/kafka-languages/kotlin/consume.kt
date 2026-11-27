package mail

fun listen(consumer: Consumer, topic: String) {
    consumer.subscribe(listOf("mail.welcome"))
    consumer.subscribe(listOf(topic))
    consumer.subscribe(Regex("mail\\..*"))
}
