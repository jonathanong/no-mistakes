import { createWorker } from "./wrappers";

createWorker("wrapper-unit", (job) => {
  switch (job.name) {
    case "aliased":
    case "unknown-options":
    case "mutated-defaults":
    case "mutated-options":
    case "mutated-update":
      return;
  }
});
createWorker("wrapper-legacy", () => {});
