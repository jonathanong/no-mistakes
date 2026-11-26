import { Worker } from "bullmq";
import { RenamedPause } from "./barrel";

export const worker = new Worker("widgets", async () => {
  throw new RenamedPause();
});
