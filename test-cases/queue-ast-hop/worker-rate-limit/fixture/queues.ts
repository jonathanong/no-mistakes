import { Queue } from "bullmq";
import { createQueue } from "./factory";

export const emails = new Queue("emails");
export const orders = createQueue("orders");
export const alerts = createQueue("alerts");
export const billing = new Queue("billing");
export const reports = createQueue("reports");
export const notices = new Queue("notices");
