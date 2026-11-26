import { Queue, Worker } from "bullmq";

const widgets = new Queue("widgets", {
  defaultJobOptions: { removeOnComplete: true },
});
new Worker("widgets", async () => {});

await widgets.add("build", {});
