import { Queue } from "bullmq";

const orders = new Queue("orders");
const method = "upsertJobScheduler";

orders.upsertJobScheduler("weekly", { every: 1000 }, {
  name: "pack",
  data: {
    delay: 1,
    jobId: "inside-data",
    parent: { id: "nested" },
    deduplication: { id: "nested" },
  },
  opts: {
    attempts: 3,
    backoff: { type: "fixed", delay: 250 },
    repeat: { every: 1000 },
  },
});

orders.upsertJobScheduler("hourly", { every: 1000 });
orders.close();
orders["close"]();
orders[method]("daily", { every: 1000 }, { name: "pack", opts: { delay: 1 } });
