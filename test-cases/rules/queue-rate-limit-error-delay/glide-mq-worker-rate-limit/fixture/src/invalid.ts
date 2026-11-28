import { GroupRateLimitError, Worker } from "glide-mq";

export function throwBareWorkerRateLimit(): never {
  throw new Worker.RateLimitError();
}

export function throwZeroDelayMs(): never {
  throw Object.assign(new Worker.RateLimitError(), { delayMs: 0 });
}

export function throwGroupWithoutDelay(): never {
  throw new GroupRateLimitError(0);
}
