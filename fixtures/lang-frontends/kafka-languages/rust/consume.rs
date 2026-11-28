fn listen(consumer: &Consumer, topic: &str) {
    consumer.subscribe(&["mail.welcome"]);
    consumer.subscribe(&[topic_for("tenant")]);
    consumer.subscribe(&[topic]);
    consumer.subscribe(Regex::new("mail\\..*").unwrap());
}

struct Consumer;

impl Consumer {
    fn subscribe(&self, _topics: &[&str]) {}
}

struct Regex;

impl Regex {
    fn new(_pattern: &str) -> Self {
        Self
    }

    fn unwrap(self) -> Self {
        self
    }
}
