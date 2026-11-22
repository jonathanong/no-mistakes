import { Queue } from "bullmq";

export const widgets = new Queue("widgets");
export const items = new Queue("items");
export const orders = new Queue("orders");
export const drafts = new Queue("drafts");
export const holds = new Queue("holds");
