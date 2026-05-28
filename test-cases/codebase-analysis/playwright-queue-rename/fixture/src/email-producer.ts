import { emailQueue } from "./queues";

export async function enqueueWelcome(userId: string) {
  await emailQueue.add("send-welcome-email", { userId });
}
