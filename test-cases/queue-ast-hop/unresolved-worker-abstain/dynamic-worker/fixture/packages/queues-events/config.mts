import { Queue } from "bullmq";

export const EVENTS_QUEUE = "events";
export const eventsQueue = new Queue(EVENTS_QUEUE);
