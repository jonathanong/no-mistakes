import { Queue } from "bullmq";
import { scheduleItems, scheduleOrders as schedulePack } from "./widgets";

const orders = new Queue("orders");

schedulePack({
  name: "pack",
  opts: { jobId: "orders" },
});

scheduleItems(orders, { every: 1000 }, {
  name: "pack",
  opts: { parent: { id: "items" } },
});
