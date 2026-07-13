import { Queue } from "bullmq";

export const emailsQueue = new Queue("ignored-emails");
