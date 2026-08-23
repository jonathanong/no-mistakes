import { emails } from "@linked/queues";

export function enqueueEmail() {
  return emails.add("sendEmail", {});
}
