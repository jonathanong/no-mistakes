import { RateLimitError } from "glide-mq";

export function triggerSuppressed(): never {
  // no-mistakes-disable-next-line queue-rate-limit-error-delay: intentional fallback
  if (Math.random() > 0.5) throw new RateLimitError();
  throw new RateLimitError(); // no-mistakes-disable-line queue-rate-limit-error-delay: intentional fallback
}
