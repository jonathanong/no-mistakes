import { enqueueBilling } from "@example/queues-mail/enqueue";

export function billAlpha() {
  enqueueBilling({ name: "charge" }, { account: "a" });
}
