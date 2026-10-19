import { createWorker } from "../lib.mts";
import { queue } from "../queues.mts";

export const renamed = createWorker("jobs", async (task) => {
  await queue.add(task.name, {}, { jobId: task.id });
  switch (task.name) {
    case "page":
      await queue.add("page", {}, { deduplication: { id: task.id } });
      return;
  }
});

export const destructured = createWorker("jobs", async ({ name, id }) => {
  switch (name) {
    case "page":
      await queue.add("page", {}, { jobId: id });
  }
});

// Shapes that must not invent a job binding.
export const laterJob = createWorker("jobs", async (ctx, job) => {
  switch (job.name) {
    case "page":
      await queue.add("page", {}, { jobId: job.id });
  }
  return ctx;
});

export const defaults = createWorker("jobs", async (task = undefined) => {
  switch (flag()) {
    case "page":
      return task;
  }
});
export const rest = createWorker("jobs", async (...jobs) => jobs);
export const listed = createWorker("jobs", async ([name]) => name);
const label = "name";

export const computed = createWorker("jobs", async ({ ["name"]: label }) => label);
export const dynamicKey = createWorker("jobs", async ({ [label]: name }) => name);
export const dataOnly = createWorker("jobs", async ({ data }) => data);
export const nested = createWorker("jobs", async ({ name: { value } }) => value);
export const fallback = createWorker("jobs", async ({ name = "page", id = "1" }) => {
  switch (name) {
    case "page":
      await queue.add("page", {}, { jobId: id });
  }
});

function flag(): string {
  return "page";
}
