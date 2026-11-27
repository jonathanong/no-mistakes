def listen(consumer, topic_name)
  consumer.subscribe("mail.welcome")
  consumer.subscribe(topic_name)
  consumer.subscribe(/mail.*/)
end
