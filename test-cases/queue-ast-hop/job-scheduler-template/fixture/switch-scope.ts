import { Queue } from "bullmq";

const orders = new Queue("orders");
const template = { name: "outer", opts: {} };
declare const route: number;

switch (route) {
  case 0:
    const template = { name: "inner", opts: { delay: 1 } };
    orders.upsertJobScheduler("body", {}, template);
    break;
}

orders.upsertJobScheduler("after", {}, template);
