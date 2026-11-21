import { Queue } from "bullmq";

export const widgets = new Queue("widgets");
export const items = new Queue("items");
export const orders = new Queue("orders");
export const holds = new Queue("holds");
export const shipments = new Queue("shipments");
export const weights = new Queue("weights");
export const credits = new Queue("credits");
export const extras = new Queue("extras");
export const drafts = new Queue("drafts");
export const tags = new Queue("tags");
export const batches = new Queue("batches");
export const parcels = new Queue("parcels");
export const labels = new Queue("labels");
export const notes = new Queue("notes");
export const catalog = new Queue("catalog");
