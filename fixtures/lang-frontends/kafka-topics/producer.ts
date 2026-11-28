export async function sendWelcome(
  producer: { send: (input: { topic: string }) => Promise<void> },
  tenant: string,
) {
  await producer.send({ topic: "mail.welcome" });
  await producer.send({ topic: "mail." + tenant });
}
