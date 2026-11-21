import { Queue } from "bullmq";

const orders = new Queue("orders");

orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  opts: {
    // no-mistakes-disable-next-line queues-check
    delay: 1,
    jobId: "kept",
  },
});
