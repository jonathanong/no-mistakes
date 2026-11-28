package mail

func listen(pattern string) {
	consumer.SubscribeTopics([]string{"mail.welcome"}, nil)
	consumer.SubscribeTopics([]string{pattern}, nil)
	consumer.SubscribeTopics([]string{"^mail.*"}, nil)
	consumer.SubscribeTopics([]string{resolveTopic("tenant")}, nil)
	consumer.Subscribe(regexp.MustCompile("mail\\..*"), nil)
}

var consumer interface {
	SubscribeTopics(topics []string, handler any)
	Subscribe(pattern any, handler any)
}

var regexp struct {
	MustCompile func(string) any
}
