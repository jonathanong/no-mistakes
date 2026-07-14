import { Worker } from "bullmq";

new Worker("visibility", async () => import("./ignored-processor"));
