import { widgets } from "../queues";

// Exempted by queues.jobStateAsRecord.allowPaths.
export async function operatorRetry(id: string) {
  const job = await widgets.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry();
}
