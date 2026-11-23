import defaultWorker, { aliasedExport, orderWorker, reexportedWorker } from "./barrel";
import * as factoryNs from "./factory";
import { sharedLimiter } from "./factory";
import * as factoryWorkers from "./factory-worker";

export async function throttleImported() {
  await reexportedWorker.rateLimit(1000);
  await aliasedExport.rateLimit(900);
  await defaultWorker.rateLimit(800);
  await orderWorker.rateLimit(700);
  await factoryWorkers.exportedWorker.rateLimit(500);
  await sharedLimiter.rateLimit(1000);
  await factoryNs.sharedLimiter.rateLimit(500);
}
