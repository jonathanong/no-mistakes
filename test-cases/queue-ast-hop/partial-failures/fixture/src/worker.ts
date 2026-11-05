import { createWorker } from "./factory";

export const worker = createWorker("healthy-queue", async (job) => {
  if (job.name === "missing-job") return;
});
