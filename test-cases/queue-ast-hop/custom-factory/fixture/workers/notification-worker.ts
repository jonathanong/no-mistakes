import { Worker } from "bullmq";

new Worker("notifications", async (job) => {
  console.log(job.data);
});
