import { Worker } from "bullmq";
import * as bullmq from "bullmq";

async function processEmail() {
  await worker.rateLimit(500);
}

export const worker = new Worker("emails", processEmail);
const alias = worker;

export async function throttleDirect() {
  await worker.rateLimit(1000);
  await alias.rateLimit(250);
  await worker["rateLimit"](200);
  await new bullmq.Worker("emails", async () => {}).rateLimit(100);
}
