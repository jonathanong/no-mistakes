export async function sendWelcome(producer: { send: (input: { topic: string }) => Promise<void> }) {
  await producer.send({ topic: "mail.welcome" });
}
