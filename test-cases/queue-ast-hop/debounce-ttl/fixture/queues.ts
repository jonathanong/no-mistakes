import { Queue } from "glide-mq";

export const alertsQueue = new Queue("alerts", {
  defaultJobOptions: {
    deduplication: { id: "alert-default", mode: "debounce", ttl: 1000 },
  },
});

export const throttleQueue = new Queue("throttle", {
  defaultJobOptions: {
    deduplication: { id: "throttle-default", mode: "throttle", ttl: 1000 },
  },
});

export const eventsQueue = new Queue("events");
