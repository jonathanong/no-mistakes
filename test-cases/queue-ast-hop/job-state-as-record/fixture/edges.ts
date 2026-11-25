import * as queues from "./barrel";
import { widgets } from "./queues";

// Reported: early-exit guard followed by aliased job.remove() and aliased state var
export async function removeAliasedJob(id: string) {
  const job = await widgets.getJob(id);
  const completed = await job?.isCompleted();
  const isDone = completed;
  if (!isDone) return;
  let pending;
  void pending;
  const sameJob = job;
  await sameJob.remove();
}

// Reported: namespace-imported re-exported queue receiver
export async function recreateViaNamespace(id: string) {
  const job = await queues.widgetQueue.getJob(id);
  if (await job?.isFailed()) await widgets.add("repair", { id });
}

// Reported: derived listing variable mapped and copied before addBulk()
export async function replayDerivedCompleted() {
  const completed = await widgets.getCompleted(0, 100);
  const copied = completed;
  const spread = [...copied];
  const batch = spread.map((job) => ({ name: job.name, data: job.data }));
  await widgets.addBulk(batch);
}

// Reported: for..of listing loop with derived payload object and spread args
export async function replayDerivedLoopPayload() {
  for (const job of await widgets.getFailed(0, 100)) {
    const payload = { ...job.data, retried: true };
    await widgets.addBulk([{ name: job.name, data: payload }]);
  }
  for (const job of await widgets.getFailed(0, 10)) {
    const args: [string, object] = [job.name, job.data];
    await widgets.add(...args);
  }
}

// Reported: chained getJob -> isFailed (no local job variable) and else branch
export async function recreateInElse(id: string) {
  if (await (await widgets.getJob(id))?.isCompleted()) {
    return;
  } else {
    await widgets.add("repair", { id });
  }
}

// Accepted: state check on one job does not guard retry on a different job
export async function differentJob(firstId: string, secondId: string) {
  const first = await widgets.getJob(firstId);
  const second = await widgets.getJob(secondId);
  if (await first?.isFailed()) await second?.retry();
}

// Accepted: delayed arrow or function callback returned from guard is not immediate
export async function delayedCallback(id: string) {
  const job = await widgets.getJob(id);
  if (await job?.isFailed()) return () => job.retry();
  if (await job?.isCompleted()) {
    return function runLater() {
      return job.remove();
    };
  }
}

// Accepted: add inside listing loop that does not use the loop job
export async function unrelatedLoopAdd() {
  for (const job of await widgets.getFailed(0, 100)) {
    console.log(job.name);
    await widgets.add("heartbeat", {});
  }
}

// Accepted: shadowed namespace binding and non-imported member receiver do not resolve
export async function shadowedNamespace(
  queues: { widgetQueue: typeof widgets },
  holder: { widgetQueue: typeof widgets },
  id: string,
) {
  const a = await queues.widgetQueue.getJob(id);
  if (await a?.isFailed()) await a.retry();
  const b = await holder.widgetQueue.getJob(id);
  if (await b?.isFailed()) await b.retry();
}
