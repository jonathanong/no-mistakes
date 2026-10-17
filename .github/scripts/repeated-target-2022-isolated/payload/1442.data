import { emailQueue } from "../queues/email";

export async function sendEmail(to: string) {
  await emailQueue.add("send", { to });
}
