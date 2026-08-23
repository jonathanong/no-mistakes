import { emailQueue } from "@queues/email";

export function enqueueB() {
  return emailQueue.add("b-job", {});
}
