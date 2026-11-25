import { enqueueMail } from "@example/queues-mail/enqueue";

const WELCOME_JOB = { name: "welcome" } as const;

export function sendWelcome() {
  enqueueMail(WELCOME_JOB, { userId: "u-1" });
}
