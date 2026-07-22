import { Queue } from "bullmq";

export const emailQueue = new Queue("b-queue");
