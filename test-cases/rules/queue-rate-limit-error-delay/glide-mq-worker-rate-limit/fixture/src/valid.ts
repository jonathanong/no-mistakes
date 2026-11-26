import { GroupRateLimitError, Worker } from "glide-mq";

export function throwWithObjectAssign(retryAfterMs: number): never {
  throw Object.assign(new Worker.RateLimitError(), { delayMs: retryAfterMs });
}

export function throwWithPropertyAssignment(retryAfterMs: number): never {
  const err = new Worker.RateLimitError();
  (err as { delayMs?: number }).delayMs = retryAfterMs;
  throw err;
}

export function throwGroupRateLimitWithDelay(retryAfterMs: number): never {
  throw new GroupRateLimitError(retryAfterMs);
}
