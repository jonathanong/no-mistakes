export function listen(consumer: { subscribe: (input: { topic: string }) => void }) {
  consumer.subscribe({ topic: "mail.welcome" });
}
