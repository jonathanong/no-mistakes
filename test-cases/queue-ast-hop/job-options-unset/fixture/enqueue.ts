import { Queue, Worker } from "bullmq";

const widgets = new Queue("widgets");
new Worker("widgets", async () => {});
declare const mystery: object;
widgets.add("paint", {}, { ...mystery });
