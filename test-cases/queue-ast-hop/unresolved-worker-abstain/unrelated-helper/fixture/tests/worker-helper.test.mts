import { Worker } from "bullmq";

export function startTestWorker() {
  return new Worker(process.env.TEST_QUEUE ?? "", async () => {});
}
