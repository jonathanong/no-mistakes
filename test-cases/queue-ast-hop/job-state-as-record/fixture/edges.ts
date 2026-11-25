import * as queues from "./barrel";
import { widgets } from "./queues";

// Reported: early-exit guard followed by aliased job.remove()
export async function removeAliasedJob(id: string) {
  const job = await widgets.getJob(id);
  const completed = await job?.isCompleted();
  if (!completed) return;
  const sameJob = job;
  await sameJob.remove();
}

// Reported: namespace-imported re-exported queue receiver
export async function recreateViaNamespace(id: string) {
  const job = await queues.widgetQueue.getJob(id);
  if (await job?.isFailed()) await widgets.add("repair", { id });
}

// Reported: derived listing variable mapped before addBulk()
export async function replayDerivedCompleted() {
  const completed = await widgets.getCompleted(0, 100);
  const batch = completed.map((job) => ({ name: job.name, data: job.data }));
  await widgets.addBulk(batch);
}

// Reported: for..of listing loop with derived payload object
export async function replayDerivedLoopPayload() {
  for (const job of await widgets.getFailed(0, 100)) {
    const payload = { ...job.data, retried: true };
    await widgets.addBulk([{ name: job.name, data: payload }]);
  }
}

// Accepted: state check on one job does not guard retry on a different job
export async function differentJob(firstId: string, secondId: string) {
  const first = await widgets.getJob(firstId);
  const second = await widgets.getJob(secondId);
  if (await first?.isFailed()) await second?.retry();
}

// Accepted: delayed callback returned from guard is not an immediate action
export async function delayedCallback(id: string) {
  const job = await widgets.getJob(id);
  if (await job?.isFailed()) return () => job.retry();
}

// Accepted: add inside listing loop that does not use the loop job
export async function unrelatedLoopAdd() {
  for (const job of await widgets.getFailed(0, 100)) {
    console.log(job.name);
    await widgets.add("heartbeat", {});
  }
}
