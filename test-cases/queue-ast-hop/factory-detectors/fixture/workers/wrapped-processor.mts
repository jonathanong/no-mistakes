import { createWorker } from "../lib.mts";
import { queue } from "../queues.mts";

// An identifier processor is declared before the worker call, so its span is
// not inside the processor argument. An id alias is not the job token.
const handler = async (j) => {
  switch (j.name) {
    case "p1":
      await queue.add("p1", {}, { jobId: j.id });
  }
};
createWorker("jobs", handler);

createWorker(
  "jobs",
  (async (j) => {
    switch (j.name) {
      case "p2":
        await queue.add("p2", {}, { jobId: j.id });
    }
  }),
);

function withRetry(processor) {
  return processor;
}
createWorker(
  "jobs",
  withRetry(async (j) => {
    switch (j.name) {
      case "p3":
        await queue.add("p3", {}, { jobId: j.id });
    }
  }),
);

createWorker(
  "jobs",
  (async (j) => {
    switch (j.name) {
      case "p8":
        await queue.add("p8", {}, { jobId: j.id });
    }
  }) as any,
);

createWorker("jobs", async (j) => {
  const id = j.id;
  switch (j.name) {
    case "p5":
      await queue.add("p5", {}, { jobId: id });
  }
});
