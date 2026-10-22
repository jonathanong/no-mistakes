import { emailsQueue } from "./queues";

type Welcome = { userId: string };

export function enqueueWelcome(data: Welcome) {
  return emailsQueue.add<Welcome>("sendWelcome", data);
}
