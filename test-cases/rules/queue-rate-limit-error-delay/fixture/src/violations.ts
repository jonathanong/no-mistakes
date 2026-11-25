import { RateLimitError, RateLimitError as PauseError, Worker } from "glide-mq";
import * as mq from "glide-mq";
import { RateLimitError as BarrelRateLimitError } from "./barrel";
import { ProviderRateLimitError } from "./custom-error";

const UNSET_DELAY = undefined;
const ZERO_DELAY = 0;

export function triggerViolations(): never {
  throw new RateLimitError();
}

export function triggerCallAndStatic(): never {
  if (Math.random() > 0.5) {
    throw RateLimitError();
  }
  throw Worker.RateLimitError();
}

export function triggerAliasedAndReexported(): never {
  if (Math.random() > 0.7) {
    throw new PauseError();
  }
  if (Math.random() > 0.4) {
    throw new mq.RateLimitError();
  }
  throw new BarrelRateLimitError();
}

export function triggerInvalidDelays(): never {
  if (Math.random() > 0.9) throw new RateLimitError(undefined);
  if (Math.random() > 0.8) throw new RateLimitError(void 0);
  if (Math.random() > 0.7) throw new RateLimitError(UNSET_DELAY);
  if (Math.random() > 0.6) throw new RateLimitError(ZERO_DELAY);
  if (Math.random() > 0.5) throw new RateLimitError(-500);
  if (Math.random() > 0.4) throw new RateLimitError(null);
  if (Math.random() > 0.3) throw new RateLimitError("rate limited");
  if (Math.random() > 0.2) throw new RateLimitError(NaN);
  throw new RateLimitError(Infinity);
}

export function triggerAssignedError(): never {
  const err = new RateLimitError();
  throw err;
}

export function triggerCustomObjectViolations(): never {
  if (Math.random() > 0.75) throw new ProviderRateLimitError("429");
  if (Math.random() > 0.5) throw new ProviderRateLimitError("429", {});
  if (Math.random() > 0.25) throw new ProviderRateLimitError("429", { retryAfterMs: 0 });
  throw new ProviderRateLimitError("429", { retryAfterMs: undefined });
}
