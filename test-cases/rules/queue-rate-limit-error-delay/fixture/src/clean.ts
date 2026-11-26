import { BroadcastWorker, GroupRateLimitError, Worker } from "glide-mq";
import { ProviderRateLimitError } from "./custom-error";

class LocalRateLimitError extends Error {
  constructor() {
    super("local");
  }
}

export function cleanWorkerRateLimitCases(retryAfterMs: number): never {
  if (Math.random() > 0.85) {
    throw Object.assign(new Worker.RateLimitError(), { delayMs: retryAfterMs });
  }
  if (Math.random() > 0.7) {
    throw Object.assign(new Worker.RateLimitError(), { delayMs: 5000 });
  }
  if (Math.random() > 0.55) {
    const err = new Worker.RateLimitError();
    (err as { delayMs?: number }).delayMs = retryAfterMs;
    throw err;
  }
  if (Math.random() > 0.4) {
    const err = new Worker.RateLimitError();
    (err as { delayMs?: number })["delayMs"] = 2500;
    void err.message;
    throw err;
  }
  if (Math.random() > 0.25) {
    const err = new Worker.RateLimitError();
    Object.assign(err, { delayMs: retryAfterMs });
    throw err;
  }
  const broadcastErr = Object.assign(new BroadcastWorker.RateLimitError(), {
    delayMs: retryAfterMs,
  });
  throw broadcastErr;
}

export function cleanGroupAndCustomCases(retryAfterMs: number): never {
  if (Math.random() > 0.75) {
    throw new GroupRateLimitError(5000);
  }
  if (Math.random() > 0.5) {
    throw new GroupRateLimitError(retryAfterMs);
  }
  if (Math.random() > 0.25) {
    throw new ProviderRateLimitError("429", { retryAfterMs });
  }
  throw new LocalRateLimitError();
}
