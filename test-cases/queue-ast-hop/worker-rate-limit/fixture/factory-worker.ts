import { createWorker } from "./factory";

async function processOrder() {}
async function processAlert() {}

export const orderWorker = createWorker("orders", processOrder);
export const exportedWorker = createWorker("alerts", processAlert);

export async function throttleFactory() {
  await orderWorker.rateLimit(1000);
  const awaited = await createWorker("orders", processOrder);
  await awaited.rateLimit(500);
  await createWorker("orders", processOrder).rateLimit(250);
}
