import { Queue, Worker } from "bullmq";
import { billingQueue } from "../src/billing";

export async function exerciseAllRulesInTest() {
  const alpha = new Queue("alpha-test");
  const beta = new Queue("beta-test");

  const alphaWorker = new Worker("alpha-test", async (job) => {
    await beta.add("step", { id: job.id });
    await alpha.add("loop", {}, { jobId: job.id });
    await alphaWorker.rateLimit(100);
  });

  const betaWorker = new Worker("beta-test", async () => {
    await alpha.add("back", {}, { jobId: "bad:id" });
  });
  void betaWorker;

  await billingQueue.add("extra-test-job", {}, { priority: 2 });
  await alpha.add("a", {}, { priority: 0 });
  await alpha.add("b", {}, { priority: 5 });

  const job = await alpha.getJob("1");
  if ((await job?.getState()) === "failed") {
    await job.retry();
  }
}
