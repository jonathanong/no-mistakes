import { mailQueue } from "@acme/queues-mail/config";

export async function sendWelcome() {
  await mailQueue.add("welcome", { userId: "u1" });
}
