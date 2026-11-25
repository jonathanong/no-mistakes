import { RateLimitError, Worker } from "glide-mq";
import { RateLimitError as OtherRateLimitError } from "other-queue-lib";
import { ProviderRateLimitError } from "./custom-error";

class LocalRateLimitError extends Error {}

function parseRetryAfter(header: string): number {
  return Number(header) * 1000;
}

export function triggerClean(retryAfterMs: number, header: string): never {
  if (retryAfterMs > 0) {
    throw new RateLimitError(retryAfterMs);
  }
  if (header.length > 0) {
    throw new RateLimitError(parseRetryAfter(header));
  }
  if (header === "static") {
    throw new RateLimitError(5000);
  }
  if (header === "worker") {
    throw Worker.RateLimitError(2500);
  }
  if (header === "custom-dynamic") {
    throw new ProviderRateLimitError("429", { retryAfterMs });
  }
  if (header === "custom-literal") {
    throw new ProviderRateLimitError("429", { retryAfterMs: 1500 });
  }
  if (header === "other-pkg") {
    throw new OtherRateLimitError();
  }
  throw new LocalRateLimitError();
}
