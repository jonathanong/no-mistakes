import { GroupRateLimitError, Worker } from "glide-mq";

export function suppressedCases(): never {
  // no-mistakes-disable-next-line queue-rate-limit-error-delay: intentional fallback
  if (Math.random() > 0.5) throw new Worker.RateLimitError();
  throw new GroupRateLimitError(); // no-mistakes-disable-line queue-rate-limit-error-delay: intentional fallback
}
