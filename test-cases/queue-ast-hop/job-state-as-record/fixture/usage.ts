import { widgets } from "./queues";

declare const db: {
  getJob: (id: string) => Promise<{ getState: () => Promise<string>; retry: () => Promise<void> } | null>;
  getFailed: (a: number, b: number) => Promise<Array<{ name: string; data: object; retry: () => Promise<void> }>>;
};
declare function pendingWidgets(store: unknown, limit: number): Promise<Array<{ id: string }>>;

// Reported: getJob -> getState guarding retry()
export async function retryIfFailed(id: string) {
  const job = await widgets.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry();
}

// Reported: getJob -> isCompleted guarding remove()
export async function cleanupCompleted(id: string) {
  const job = await widgets.getJob(id);
  if (await job?.isCompleted()) {
    await job.remove();
  }
}

// Reported: getJob -> isFailed early return guarding new add()
export async function requeueAfterGuard(id: string) {
  const job = await widgets.getJob(id);
  const failed = await job?.isFailed();
  if (!failed) return;
  await widgets.add("build", { id });
}

// Reported: getFailed feeding add()
export async function replayFailed() {
  for (const job of await widgets.getFailed(0, 100)) {
    await widgets.add(job.name, job.data);
  }
}

// Reported: getCompleted feeding addBulk()
export async function replayCompleted() {
  const done = await widgets.getCompleted(0, 50);
  await widgets.addBulk(done.map((job) => ({ name: job.name, data: job.data })));
}

// Reported: getJobs(['failed']) feeding retry() via map
export async function retryFromGetJobs() {
  const jobs = await widgets.getJobs(["waiting", "failed"]);
  await Promise.all(jobs.map((job) => job.retry()));
}

// Accepted: database decides what is pending
export async function enqueueFromDb() {
  for (const row of await pendingWidgets(db, 100)) {
    await widgets.add("build", { id: row.id });
  }
}

// Accepted: getJobs only inspects waiting/active (not failed/completed)
export async function inspectWaiting() {
  for (const job of await widgets.getJobs(["waiting", "active"])) {
    await widgets.add(job.name, job.data);
  }
}

// Accepted: read-only state inspection does not guard retry/remove/add
export async function readStateOnly(id: string) {
  const job = await widgets.getJob(id);
  const state = await job?.getState();
  return state;
}

// Accepted: non-queue Set.add inside state branch does not count as queue add
export async function recordSeenIds(id: string, seen: Set<string>) {
  const job = await widgets.getJob(id);
  if ((await job?.getState()) === "failed") {
    seen.add(id);
  }
}

// Abstained: receiver does not resolve to a configured queue binding
export async function nonQueueReceiver(id: string) {
  const job = await db.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry();
  for (const item of await db.getFailed(0, 10)) {
    await item.retry();
  }
}

// Suppressed via queues-check directive
export async function suppressedRetry(id: string) {
  const job = await widgets.getJob(id);
  // no-mistakes-disable-next-line queues-check
  if ((await job?.getState()) === "failed") await job.retry();
}
