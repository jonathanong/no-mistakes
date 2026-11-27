package mail

func publish() {
	_ = &Writer{Topic: "mail.welcome"}
}
