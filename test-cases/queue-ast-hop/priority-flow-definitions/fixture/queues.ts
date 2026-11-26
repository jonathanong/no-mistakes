import { Queue, Worker } from "bullmq";

export const orders = new Queue("orders");
export const reports = new Queue("reports");
new Worker("orders", async () => {});
new Worker("reports", async () => {});
