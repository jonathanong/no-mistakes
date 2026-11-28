package mail

func listen() {
	consumer.SubscribeTopics([]string{"mail.welcome"}, nil)
}
