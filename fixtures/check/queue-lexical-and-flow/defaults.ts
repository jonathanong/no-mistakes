import { Queue, Worker } from "bullmq";

const left = { ...right, defaultJobOptions: { priority: 1 } };
const right = { ...left };
const queue = new Queue("defaults", { ...left, ...right });
new Worker("defaults", async () => {});

queue.add("with-default", {}, {});
