import { emailsQueue } from "@queues/emails";

export function enqueueWelcome(userId: string) {
  return emailsQueue.add("sendWelcome", { userId });
}
