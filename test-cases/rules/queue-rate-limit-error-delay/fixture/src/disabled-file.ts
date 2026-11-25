// no-mistakes-disable-file queue-rate-limit-error-delay: test harness helper
import { RateLimitError } from "glide-mq";

export function triggerDisabled(): never {
  throw new RateLimitError();
}
