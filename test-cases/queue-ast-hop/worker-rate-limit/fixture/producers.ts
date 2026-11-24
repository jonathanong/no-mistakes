import { alerts, billing, emails, notices, orders, reports } from "./queues";

export function enqueueAll() {
  return Promise.all([
    emails.add("send", {}),
    orders.add("fulfill", {}),
    alerts.add("notify", {}),
    billing.add("charge", {}),
    reports.add("build", {}),
    notices.add("deliver", {}),
  ]);
}
