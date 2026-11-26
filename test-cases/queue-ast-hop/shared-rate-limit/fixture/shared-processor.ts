import { RateLimitError, Worker } from "bullmq";

async function handle() {
  throw new RateLimitError();
}

export const widgetsWorker = new Worker("widgets", handle);
export const itemsWorker = new Worker("items", handle);
