import { Worker } from "glide-mq";

export function excludedCall(): never {
  throw new Worker.RateLimitError();
}
