import { RateLimitError } from "glide-mq";

export function excludedCall(): never {
  throw new RateLimitError();
}
