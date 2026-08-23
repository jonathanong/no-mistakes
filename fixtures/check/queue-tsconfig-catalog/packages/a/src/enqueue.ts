import { emailQueue } from "@queues/email";

export function enqueueA() {
  return emailQueue.add("a-job", {});
}
