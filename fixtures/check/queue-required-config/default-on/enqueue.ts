import { Queue, Worker } from "bullmq";

const dispatch = new Queue("dispatch");
dispatch.add("render", {}, {});
new Worker("dispatch", async () => {});
