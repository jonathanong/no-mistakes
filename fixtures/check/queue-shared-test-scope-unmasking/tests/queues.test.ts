import { batchQueue } from "../src/batch";
import { dispatchQueue } from "../src/dispatch";
import { triageQueue } from "../src/triage";

export async function exerciseQueuesInTest(
  jobName: string,
  priority: number,
  jobs: Array<{ name: string; data: Record<string, unknown> }>,
) {
  await dispatchQueue.add(jobName, {});
  await triageQueue.add("normal", {}, { priority });
  await batchQueue.addBulk(jobs);
}
