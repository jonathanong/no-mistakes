import { Worker } from "bullmq";
import * as processors from "@linked/processors";

new Worker("emails", async (job) => processors[job.name]());
