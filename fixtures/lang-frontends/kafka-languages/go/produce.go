package mail

func publish(topic string) {
	_ = &Writer{Topic: "mail.welcome"}
	_ = ProducerMessage{Topic: "mail.welcome"}
	_ = &Writer{Topic: topic}
	_ = &Writer{Topic: "mail." + tenant}
	_ = ProducerMessage{Topic: "prefix." + tenant}
}

type Writer struct {
	Topic string
}

type ProducerMessage struct {
	Topic string
}
