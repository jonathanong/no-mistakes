def publish(producer, topic_name)
  producer.produce(payload, topic: "mail.welcome")
  producer.produce(payload, topic: topic_name)
  producer.produce(payload, topic: "mail." + topic_name)
  producer.produce(payload, topic: "mail.#{topic_name}")
end
