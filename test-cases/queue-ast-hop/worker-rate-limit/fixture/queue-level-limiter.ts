import { Worker } from "bullmq";

export const billingWorker = new Worker("billing", async () => {}, {
  limiter: { max: 10, duration: 1000 },
});
