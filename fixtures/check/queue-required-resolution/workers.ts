import { Worker } from "bullmq";

new Worker("shared", async () => {});
new Worker("local", async () => {});
