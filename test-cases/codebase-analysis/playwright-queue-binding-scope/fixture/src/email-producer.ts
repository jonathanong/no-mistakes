import { emailQueue } from "./queues";

export async function enqueueSync(userId: string) {
  await emailQueue.add("sync", { userId });
}
