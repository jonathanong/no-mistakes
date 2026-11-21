// no-mistakes-disable-file queues-check
import { Queue } from "bullmq";

const orders = new Queue("orders");

orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  opts: { delay: 1 },
});
