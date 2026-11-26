import { FlowProducer } from "bullmq";
import { orders, reports } from "./queues";

orders.add("plain", {});
reports.add("plain", {});
const flow = new FlowProducer();
flow.add({ name: "rush", queueName: "orders", opts: { priority: 2 } });
flow.add({ name: "rush", queueName: "reports", opts: { priority: 2 } });
