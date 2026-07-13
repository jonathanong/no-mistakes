import { Queue } from "bullmq";

export const emailsQueue = new Queue("visible-emails");
