package mail

func publish(topic string) {
	_ = &Writer{Topic: "mail.welcome"}
	_ = ProducerMessage{Topic: "mail.welcome"}
	_ = &Writer{Topic: topic}
}

type Writer struct {
	Topic string
}

type ProducerMessage struct {
	Topic string
}
