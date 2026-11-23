import { createWorker } from "./wrappers";

createWorker("partial-jobs", async job => {
  switch (job.name) {
    case "pending":
      return;
    case "known":
      return;
  }
});

createWorker("partial-options", async job => {
  switch (job.name) {
    case "defaults":
      return;
    case "zero":
      return;
  }
});
