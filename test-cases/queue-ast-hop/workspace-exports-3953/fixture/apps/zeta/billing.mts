import { enqueueBilling } from "@example/queues-mail/enqueue";

export function billZeta() {
  enqueueBilling({ name: "charge" }, { account: "z" });
}
