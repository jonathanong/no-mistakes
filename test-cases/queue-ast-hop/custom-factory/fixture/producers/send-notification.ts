import { notificationsQueue } from "../queues/notifications";
export async function sendNotification(data: Record<string, unknown>) {
  await notificationsQueue.add("notify", data);
}
