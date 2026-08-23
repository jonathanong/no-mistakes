export function listen(consumer: {
  subscribe: (input: { topic: string } | string[]) => void;
}) {
  consumer.subscribe({ fromBeginning: true, topic: "mail.welcome" });
  consumer.subscribe(["orders", "payments"]);
}
