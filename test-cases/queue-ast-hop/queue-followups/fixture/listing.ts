import { Queue } from "bullmq";

const tracked = new Queue("tracked", { defaultJobOptions: { removeOnFail: true } });

export async function unrelatedActions() {
  for (const job of await tracked.getFailed(0, 10)) {
    await tracked.add("audit", {});
  }
}

export async function shadowedItem() {
  for (const job of await tracked.getFailed(0, 10)) {
    const job = { id: "local" };
    await tracked.add("audit", job);
  }
}

export async function usesIndexOnly() {
  (await tracked.getFailed(0, 10)).forEach((job, index) => tracked.add("audit", { index }));
}

export async function usesListingItem() {
  for (const job of await tracked.getFailed(0, 10)) {
    await tracked.add("retry", job.data);
  }
}

export async function usesDestructuredItem() {
  (await tracked.getCompleted(0, 10)).forEach(({ data }) =>
    tracked.addBulk([{ name: "retry", data }]),
  );
}

export async function usesLocalAlias() {
  for (const job of await tracked.getFailed(0, 10)) {
    const payload = job.data;
    await tracked.add("retry", payload);
  }
}

declare function normalize(data: object): object;
declare const fallback: { retry(): Promise<void> };

export async function usesCallResult() {
  for (const job of await tracked.getFailed(0, 10)) {
    const payload = normalize(job.data);
    await tracked.add("retry", payload);
  }
}

export async function usesCompoundAlias() {
  for (const job of await tracked.getFailed(0, 10)) {
    const payload = { data: job.data };
    await tracked.add("retry", payload);
  }
}

export async function usesDestructuredAlias() {
  for (const job of await tracked.getFailed(0, 10)) {
    const { data: payload } = job;
    await tracked.add("retry", payload);
  }
}

export async function retriesDestructuredBinding() {
  for (const { data: job } of await tracked.getFailed(0, 10)) {
    await job.retry();
  }
}

export async function retriesDefaultedBinding() {
  (await tracked.getFailed(0, 10)).forEach(({ data: job } = { data: fallback }) => job.retry());
}

export async function prefersProvenUse() {
  (await tracked.getFailed(0, 10)).forEach((job) => {
    const payload = normalize(job.data);
    tracked.add("retry", payload);
    tracked.add("retry", job.data);
  });
}

export async function unrelatedCallResult() {
  for (const job of await tracked.getFailed(0, 10)) {
    const payload = normalize({});
    await tracked.add("audit", payload);
  }
}

export async function usesCallInsideCompoundAlias() {
  for (const job of await tracked.getFailed(0, 10)) {
    const payload = { data: normalize(job.data) };
    await tracked.add("retry", payload);
  }
}
