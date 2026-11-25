import { Queue, Worker } from "bullmq";

export const widgets = new Queue("widgets");
new Worker("widgets", async () => {});
