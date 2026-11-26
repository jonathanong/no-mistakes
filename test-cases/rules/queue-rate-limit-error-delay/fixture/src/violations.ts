import {
  BroadcastWorker,
  GroupRateLimitError,
  GroupRateLimitError as PauseGroupError,
  Worker,
  Worker as QueueWorker,
} from "glide-mq";
import * as mq from "glide-mq";
import { GroupRateLimitError as BarrelGroupRateLimitError } from "./barrel";
import { ProviderRateLimitError } from "./custom-error";

const UNSET_DELAY = undefined;
const ZERO_DELAY = 0;

export function triggerWorkerViolations(): never {
  if (Math.random() > 0.9) throw new Worker.RateLimitError();
  if (Math.random() > 0.8) throw Worker.RateLimitError();
  if (Math.random() > 0.7) throw new QueueWorker.RateLimitError();
  if (Math.random() > 0.6) throw new mq.Worker.RateLimitError();
  if (Math.random() > 0.4) throw new BroadcastWorker.RateLimitError();
  const bare = new Worker.RateLimitError();
  if (Math.random() > 0.3) throw bare;
  if (Math.random() > 0.2) throw Object.assign(new Worker.RateLimitError(), {});
  if (Math.random() > 0.1) throw Object.assign(new Worker.RateLimitError(), { delayMs: 0 });
  throw Object.assign(new Worker.RateLimitError(), { delayMs: undefined });
}

export function triggerWorkerAssignedInvalidDelays(): never {
  if (Math.random() > 0.5) {
    const neg = new Worker.RateLimitError();
    (neg as { delayMs?: number }).delayMs = -100;
    throw neg;
  }
  const nanErr = new Worker.RateLimitError();
  (nanErr as { delayMs?: number }).delayMs = NaN;
  throw nanErr;
}

export function triggerGroupAliasedAndReexported(): never {
  if (Math.random() > 0.7) throw new PauseGroupError();
  if (Math.random() > 0.4) throw new mq.GroupRateLimitError();
  throw new BarrelGroupRateLimitError();
}

export function triggerGroupInvalidDelays(): never {
  if (Math.random() > 0.9) throw new GroupRateLimitError();
  if (Math.random() > 0.8) throw new GroupRateLimitError(undefined);
  if (Math.random() > 0.7) throw new GroupRateLimitError(void 0);
  if (Math.random() > 0.6) throw new GroupRateLimitError(UNSET_DELAY);
  if (Math.random() > 0.5) throw new GroupRateLimitError(ZERO_DELAY);
  if (Math.random() > 0.4) throw new GroupRateLimitError(-500);
  if (Math.random() > 0.3) throw new GroupRateLimitError(null);
  if (Math.random() > 0.2) throw new GroupRateLimitError("rate limited");
  if (Math.random() > 0.1) throw new GroupRateLimitError(NaN);
  throw new GroupRateLimitError(Infinity);
}

export function triggerCustomObjectViolations(): never {
  if (Math.random() > 0.75) throw new ProviderRateLimitError("429");
  if (Math.random() > 0.5) throw new ProviderRateLimitError("429", {});
  if (Math.random() > 0.25) throw new ProviderRateLimitError("429", { retryAfterMs: 0 });
  throw new ProviderRateLimitError("429", { retryAfterMs: undefined });
}
