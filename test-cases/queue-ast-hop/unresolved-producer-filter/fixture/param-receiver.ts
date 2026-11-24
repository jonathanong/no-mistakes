import type { Queue } from "bullmq";

export async function send(queue: Queue, payload: unknown): Promise<void> {
  await queue.add("send-email", payload);
}
