class Publisher {
    void publish(String payload) {
        kafkaTemplate.send("mail.welcome", payload);
    }
}
