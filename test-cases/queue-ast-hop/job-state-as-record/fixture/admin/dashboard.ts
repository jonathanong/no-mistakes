import * as queuesNs from "../queues";
import { widgets } from "../queues";

declare const states: string[];
declare const otherObj: { retry: () => void; getState: () => string; add: (x: unknown) => void };
declare function noop(item: unknown): void;

// Exempted by queues.jobStateAsRecord.allowPaths.
export async function operatorRetry(id: string) {
  const job = await widgets.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry();
}

export async function operatorVariants(id: string) {
  let uninit;
  void uninit;
  const job = await widgets?.getJob(id);
  const jobId = job?.id;
  void jobId;
  const state = await job?.getState();
  void otherObj.getState();
  // Ternary guard (both consequent and alternate forms)
  const _t = state === "failed" ? job?.retry() : null;
  const _t2 = state !== "failed" ? null : job?.retry();
  void _t;
  void _t2;

  // Switch guard + non-state switch
  switch (state) {
    case "failed":
      await job?.retry();
      await widgets.add("requeued", { id });
      break;
  }
  switch (id) {
    default:
      break;
  }

  // Two actions in one branch + non-matching retry/add + non-exit single-stmt if
  if (state === "failed") {
    otherObj.retry();
    otherObj.add(id);
    await job?.retry();
    await job?.remove();
  }
  if (state === "failed") void id;
  if (state === "failed") await job?.retry(); if (state === "failed") await job?.retry();

  // Alternate branch + block early exit
  if (state !== "failed") {
    // no-op
  } else {
    await job?.retry();
  }
  if (!state) {
    return;
  }
  await widgets.addBulk([{ name: "requeued", data: { id } }]);
  await widgets.add("plain", { id });
  otherObj.add(states);

  // Chained getJob().getState() + namespace receiver
  if ((await queuesNs.widgets.getJob(id))?.getState()) {
    await widgets.add("ns", { id });
  }

  // Listing callbacks (block arrow, function expr, no-action arrow, identifier) and getJobs variants
  const spreadList = await widgets.getJobs(...(states as [string[]]));
  void spreadList;
  const failed = await widgets.getJobs([...states, "failed"]);
  failed.forEach((item) => {
    void item.retry();
  });
  failed.forEach((item) => {
    void item;
  });
  failed.forEach(noop);
  failed.map(function (item) {
    return item.retry();
  });
  const completed = await widgets.getJobs("completed");
  for (const { name } of completed) {
    void name;
  }
  let current;
  for (current of completed) {
    void current;
  }
  {
    const nested = await widgets.getJob(id);
    if (await nested?.isCompleted()) {
      return widgets.add("after-block", { id });
    }
  }
}

export const exportedCheck = widgets.getFailed(0, 5);
