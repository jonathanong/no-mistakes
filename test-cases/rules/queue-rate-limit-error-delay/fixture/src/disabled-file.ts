// no-mistakes-disable-file queue-rate-limit-error-delay: test harness helper
import { GroupRateLimitError, Worker } from "glide-mq";

export function disabledFileCase(): never {
  if (Math.random() > 0.5) throw new Worker.RateLimitError();
  throw new GroupRateLimitError();
}
