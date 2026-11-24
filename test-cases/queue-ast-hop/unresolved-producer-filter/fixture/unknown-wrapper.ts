import { enqueueDynamic } from "./wrapper-factory";

export async function dispatch(queueName: string, payload: unknown): Promise<void> {
  await enqueueDynamic({ queueName, jobName: "send-email" }, payload);
}
