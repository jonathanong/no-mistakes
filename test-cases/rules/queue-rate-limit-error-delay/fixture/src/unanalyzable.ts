import { GroupRateLimitError, Worker } from "glide-mq";
import { Worker as BarrelWorker } from "./barrel";
import { ProviderRateLimitError as FactoryProviderRateLimitError } from "./factory-error";
import { GroupRateLimitError as StarGroupRateLimitError } from "./star-barrel";
import { ProviderRateLimitError, type ProviderLimitOptions } from "./custom-error";

declare function attachDelay(err: Error): void;
declare function getErr(): new () => Error;

export function triggerWorkerUnanalyzable(
  flag: boolean,
  extra: Record<string, unknown>,
  key: string,
): Error {
  if (flag && Math.random() > 0.8) {
    throw Object.assign(new Worker.RateLimitError(), extra);
  }
  if (flag && Math.random() > 0.6) {
    throw Object.assign(new Worker.RateLimitError(), { ...extra });
  }
  if (flag && Math.random() > 0.4) {
    const conditional = new Worker.RateLimitError();
    if (flag) {
      (conditional as { delayMs?: number }).delayMs = 5000;
    }
    throw conditional;
  }
  if (flag && Math.random() > 0.2) {
    const escaped = new Worker.RateLimitError();
    attachDelay(escaped);
    throw escaped;
  }
  if (flag) {
    const computed = new Worker.RateLimitError();
    (computed as unknown as Record<string, unknown>)[key] = 5000;
    throw computed;
  }
  return new Worker.RateLimitError();
}

export function triggerSpread(args: [number?]): never {
  throw new GroupRateLimitError(...args);
}

export function triggerStarReexport(): never {
  if (Math.random() > 0.5) {
    throw new BarrelWorker.RateLimitError();
  }
  throw new StarGroupRateLimitError();
}

export function triggerOpaqueOptions(opts: ProviderLimitOptions): never {
  if (Math.random() > 0.5) {
    throw new ProviderRateLimitError("429", opts);
  }
  throw new ProviderRateLimitError("429", { ...opts });
}

export function triggerFactoryBuiltErrors(): never {
  if (Math.random() > 0.5) {
    throw new FactoryProviderRateLimitError("429");
  }
  const Err = getErr();
  throw new Err();
}

export function triggerUnresolvedCallee(dynamicPkg: {
  GroupRateLimitError: new (delayMs?: number) => Error;
}): never {
  if (Math.random() > 0.5) {
    throw new dynamicPkg.GroupRateLimitError(5000);
  }
  throw new dynamicPkg.GroupRateLimitError();
}
