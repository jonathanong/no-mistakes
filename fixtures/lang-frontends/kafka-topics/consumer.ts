export function listen(
  consumer: {
    subscribe: (input: { topic: string } | string[]) => void;
  },
  name: string,
) {
  consumer.subscribe({ fromBeginning: true, topic: "mail.welcome" });
  consumer.subscribe({ topic: "mail." + name });
  consumer.subscribe(["orders", "payments"]);
}
