import { Queue } from "bullmq";

export const ordersQueue = new Queue("orders");
