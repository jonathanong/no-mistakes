import { FlowProducer, Queue, Worker } from "bullmq";

export const widgets = new Queue("widgets");
const items = new Queue("items");
const orders = new Queue("orders");
const flow = new FlowProducer();

export const widgetWorker = new Worker("widgets", async () => {});
export const itemWorker = new Worker("items", async () => {});
export const orderWorker = new Worker("orders", async () => {});

function dynamic(): string {
  return "generated";
}

export function enqueue() {
  const id = dynamic();
  const widgetId = "user:1";
  const base = {};
  return Promise.all([
    widgets.add("place", {}, { jobId: "user:1" }),
    widgets.add("place", {}, { jobId: `order-${id}:retry` }),
    widgets.add("place", {}, { jobId: "a{b}" }),
    widgets.add("place", {}, { jobId: "a}b" }),
    widgets.add("place", {}, { jobId: "user-1" }),
    widgets.add("place", {}, { jobId: id }),
    widgets.add("place", {}, { jobId: `order-${id}` }),
    widgets.add("place", {}, { jobId: "user\u0001" }),
    widgets.add("place", {}, { jobId: "a:{b}" }),
    widgets.add("place", {}, { jobId: "a:{\u0001" }),
    widgets.add("place", {}, { jobId: widgetId }),
    widgets.add("place", {}, { ...base, jobId: "spread:1" }),
    widgets.add("place", {}, { jobId: dynamic() }),
    widgets.add("place", {}, id),
    // no-mistakes-disable-next-line queues-check
    widgets.add("place", {}, { jobId: "skip:me" }),
    // no-mistakes-disable-next-line
    widgets.add("place", {}, { jobId: "bare:id" }),
    items.addBulk([
      { name: "pack", data: {}, opts: { jobId: "item:1" } },
      { name: "pack", data: {}, opts: { jobId: "item-1" } },
    ]),
    orders.upsertJobScheduler("nightly", { pattern: "0 0 * * *" }),
    orders.upsertJobScheduler(
      "nightly",
      { pattern: "0 0 * * *" },
      { name: "sweep", data: {}, opts: { jobId: "order:nightly" } },
    ),
    flow.add(id),
    flow.add({
      name: "assemble",
      queueName: "orders",
      data: {},
      opts: { jobId: "order:root" },
      children: id,
    }),
    flow.add({
      name: "assemble",
      queueName: "orders",
      data: {},
      opts: { jobId: "order:root" },
      children: [
        null,
        {
          name: "pack",
          queueName: "items",
          data: {},
          opts: { jobId: "item{child}" },
        },
      ],
    }),
  ]);
}
