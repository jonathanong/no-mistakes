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
