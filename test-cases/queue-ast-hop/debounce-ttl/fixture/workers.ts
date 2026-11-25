import { Worker } from "glide-mq";

export const alertsWorker = new Worker("alerts", async () => {});
export const throttleWorker = new Worker("throttle", async () => {});
export const eventsWorker = new Worker("events", async () => {});
