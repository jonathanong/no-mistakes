import { RateLimitError } from "glide-mq";
import { RateLimitError as StarRateLimitError } from "./star-barrel";
import { ProviderRateLimitError, type ProviderLimitOptions } from "./custom-error";

export function triggerSpread(args: [number?]): never {
  throw new RateLimitError(...args);
}

export function triggerStarReexport(): never {
  throw new StarRateLimitError();
}

export function triggerOpaqueOptions(opts: ProviderLimitOptions): never {
  if (Math.random() > 0.5) {
    throw new ProviderRateLimitError("429", opts);
  }
  throw new ProviderRateLimitError("429", { ...opts });
}

export function triggerUnresolvedCallee(dynamicPkg: {
  RateLimitError: new (delayMs?: number) => Error;
}): never {
  if (Math.random() > 0.5) {
    throw new dynamicPkg.RateLimitError(5000);
  }
  throw new dynamicPkg.RateLimitError();
}
