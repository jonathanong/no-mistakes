package mail

fun listen(consumer: Consumer, topic: String) {
    consumer.subscribe(listOf("mail.welcome"))
    consumer.subscribe(listOf("mail.$tenant"))
    consumer.subscribe(listOf(topic))
    consumer.subscribe(listOf(resolveTopic("tenant")))
    consumer.subscribe(listOf("^mail.*"))
    consumer.subscribe(Regex("mail\\..*"))
}
