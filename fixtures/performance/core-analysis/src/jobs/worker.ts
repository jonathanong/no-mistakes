import { Worker } from "bullmq";

new Worker("email", async (job) => job.data);
