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

// Reported: alias reassigned to tracked job inside guard block
export async function retryReassignedAlias(firstId: string, secondId: string) {
  const first = await widgets.getJob(firstId);
  const second = await widgets.getJob(secondId);
  if (await first?.isFailed()) {
    let target = second;
    target = first;
    await target?.retry();
  }
}

// Reported: namespace-imported re-exported queue receiver
export async function recreateViaNamespace(id: string) {
  const job = await queues.widgetQueue.getJob(id);
  if (await job?.isFailed()) await widgets.add("repair", { id });
}

// Reported: derived listing variable mapped, copied, concat'd, and spread before addBulk()
export async function replayDerivedCompleted() {
  const completed = await widgets.getCompleted(0, 100);
  const copied = completed;
  const combined = ([] as typeof copied).concat(copied);
  const spread = [...combined, ...completed];
  const batch = spread.map((job) => ({ name: job.name, data: job.data }));
  await widgets.addBulk(...([batch, batch] as [typeof batch, typeof batch]));
}

// Reported: for..of listing loop with derived payload object, destructured field, and spread args
export async function replayDerivedLoopPayload() {
  for (const job of await widgets.getFailed(0, 100)) {
    const payload = { ...job.data, retried: true };
    await widgets.addBulk([{ name: "repair", data: payload }]);
  }
  for (const { data: item = {} } of await widgets.getFailed(0, 10)) {
    await widgets.add("repair", item);
  }
  for (const job of await widgets.getFailed(0, 10)) {
    const args: [string, object] = [job.name, job.data];
    await widgets.add(...args);
  }
  for (const job of await widgets.getFailed(0, 10)) {
    await widgets.add("repair", {
      nums: [1].map(() => job.data),
    });
  }
}

// Reported: chained getJob -> isFailed (no local job variable), else branch, and IIFE
export async function recreateInElse(id: string) {
  if (await (await widgets.getJob(id))?.isCompleted()) {
    return;
  } else {
    await widgets.add("repair", { id });
  }
  const job = await widgets.getJob(id);
  if (await job?.isFailed()) await (async () => job.retry())();
  if (await job?.isCompleted()) {
    await (function () {
      return job.remove();
    })();
  }
}

// Reported: callback third parameter (source array) feeding addBulk, function callback, switch, and arrow
export async function replayViaArrayParam() {
  const failed = await widgets.getFailed(0, 10);
  failed.forEach((_job, _index, all) => {
    void widgets.addBulk(all.map((j) => ({ name: j.name, data: j.data })));
  });
  failed.forEach(function (job) {
    void job.retry();
  });
  failed.forEach(({ data }) => {
    void widgets.add("repair", data);
  });
  failed.forEach(console.log);
}

export const switchAndArrow = async (id: string) => {
  const job = await widgets.getJob(id);
  const { length } = [id];
  const box = { count: length };
  box.count = 1;
  void ((await job?.isFailed()) ? job.retry() : null);
  switch (await job?.getState()) {
    case "failed":
      await job.retry();
      break;
  }
};

export const topExportCheck = widgets.getFailed(0, 1);

// Accepted: state check on one job does not guard retry on a different job or reassigned/scoped alias
export async function differentJob(firstId: string, secondId: string) {
  const first = await widgets.getJob(firstId);
  const second = await widgets.getJob(secondId);
  if (await first?.isFailed()) await second?.retry();
  let alias = first;
  const failed = await first?.isFailed();
  alias = second;
  if (failed) await alias?.retry();
  const candidate = second;
  if (failed) {
    {
      const candidate = first;
      void candidate;
    }
    for (let candidate = first; candidate; ) {
      break;
    }
    for (const candidate of [first]) {
      void candidate;
    }
    for (const candidate in { first }) {
      void candidate;
    }
    switch (secondId) {
      case "a":
        const candidate = first;
        void candidate;
        break;
    }
    for (const first of [second]) {
      await first?.retry();
    }
    try {
      throw second;
    } catch (first) {
      await (first as typeof second)?.retry();
    }
    let rebound = first;
    rebound = second;
    await rebound?.retry();
    await ((first, ...rest: unknown[]) => {
      void rest;
      return first?.retry();
    })(second);
    await candidate?.retry();
  }
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

// Accepted: add inside listing loop/callback that does not use the loop job
export async function unrelatedLoopAdd() {
  for (const job of await widgets.getFailed(0, 100)) {
    console.log(job.name);
    await widgets.add("heartbeat", {
      nums: [1].map((job) => job + 1),
      other: [1].map(function (job) {
        return job + 1;
      }),
    });
  }
  for (const job of await widgets.getFailed(0, 10)) {
    await widgets.add("heartbeat", {
      nums: [1].map(() => {
        const job = 1;
        return job + 1;
      }),
    });
  }
  (await widgets.getFailed(0, 10)).forEach((_job, index) => {
    void widgets.add("heartbeat", { index });
  });
}

// Accepted: shadowed namespace binding and non-imported/call member receivers do not resolve
export async function shadowedNamespace(
  queues: { widgetQueue: typeof widgets },
  holder: { widgetQueue: typeof widgets },
  getHolder: () => { widgetQueue: typeof widgets; retry: () => Promise<void>; add: (n: string, d: object) => Promise<void> },
  noop: () => void,
  id: string,
) {
  const a = await queues.widgetQueue.getJob(id);
  if (await a?.isFailed()) await a.retry();
  const b = await holder.widgetQueue.getJob(id);
  if (await b?.isFailed()) await b.retry();
  const c = await getHolder().widgetQueue.getJob(id);
  if (await c?.isFailed()) {
    noop();
    await getHolder().retry();
    await getHolder().add("noop", {});
  }
}
