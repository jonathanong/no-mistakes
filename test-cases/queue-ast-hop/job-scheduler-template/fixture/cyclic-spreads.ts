import { Queue } from "bullmq";

const orders = new Queue("orders");
const self = { opts: { delay: 1 }, ...self };
const mutualA = { opts: { delay: 2 }, ...mutualB };
const mutualB = { ...mutualA };

const shared = { name: "shared" };
const left = { ...shared };
const right = { ...shared, opts: { delay: 3 } };
orders.upsertJobScheduler("self", {}, self);
orders.upsertJobScheduler("mutual", {}, mutualA);
orders.upsertJobScheduler("mutual-opts", {}, { opts: mutualA });
orders.upsertJobScheduler("shared", {}, { ...left, ...right });
