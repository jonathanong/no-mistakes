import { Worker } from "bullmq";

export const worker = new Worker("items", async () => {
  await worker["rateLimit"](1000);
});
