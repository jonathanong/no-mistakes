import { RateLimitError, Worker } from "bullmq";

declare const ctor: { [name: string]: new () => Error };

export function outside(worker: { rateLimit(ms: number): Promise<void> }) {
  throw new RateLimitError();
  return worker.rateLimit(1);
}

function helper() {
  return 1;
}

export const worker = new Worker("orders", async () => {
  helper();
  throw "skip";
  throw new Error("skip");
  throw new ctor["RateLimitError"]();
  await worker.rateLimit(1);
  throw new RateLimitError();
});
