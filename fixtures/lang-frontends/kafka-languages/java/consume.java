package mail;

class Listen {
  void listen(Consumer consumer, String topic) {
    consumer.subscribe(List.of("mail.welcome"));
    consumer.subscribe(Arrays.asList("mail.welcome"));
    consumer.subscribe(Collections.singletonList("mail.welcome"));
    consumer.subscribe(List.of(topic));
    consumer.subscribe(Pattern.compile("mail\\..*"));
  }
}
