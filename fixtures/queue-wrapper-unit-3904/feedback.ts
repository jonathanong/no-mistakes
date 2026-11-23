import { createWorker, makeEnqueue } from "./wrappers";

createWorker("wrapper-unit", (job: { name: string; id?: string }) => {
  switch (job.name) {
    case "aliased": {
      const send = makeEnqueue({ name: "aliased" }, {
        attempts: 2, removeOnComplete: true, priority: 3,
      });
      send({}, { deduplication: { id: job.id } });
    }
  }
});
