import { enqueueConfigured } from "./factory";
import { jobs } from "./scoped-enqueue";

enqueueConfigured(jobs, "configured", { jobId: "configured:1" });
