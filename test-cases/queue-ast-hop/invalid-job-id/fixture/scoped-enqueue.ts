import { Queue, Worker } from "bullmq";

export const jobs = new Queue("scoped");
export const worker = new Worker("scoped", async () => {});
const id = "outer:1";

export function first() {
  const id = "inner{1}";
  jobs.add("inner", {}, { jobId: id });
}

export function shadow(id: string) {
  jobs.add("parameter", {}, { jobId: id });
}

export function mutable(value: string) {
  let id = "was:literal";
  id = value;
  jobs.add("mutable", {}, { jobId: id });
}

jobs.add("outer", {}, { jobId: id });

const registry = {
  add(_name: string, _value: unknown, _opts: object) {},
  addBulk(_items: object[]) {},
  upsertJobScheduler(_name: string, _schedule: object, _template: object) {},
};
registry.add("unrelated-add", {}, { jobId: "registry:1" });
registry.addBulk([{ name: "unrelated-bulk", data: {}, opts: { jobId: "registry:2" } }]);
declare function schedulerOptions(): object;
registry.upsertJobScheduler("daily", {}, { name: "unrelated-scheduler", opts: { ...schedulerOptions(), jobId: "registry:3" } });

export function shadowReceiver(jobs: typeof registry) {
  jobs.add("unrelated-shadow", {}, { jobId: "registry:4" });
}

export function late() {
  lateQueue.add("late", {}, { jobId: lateId });
}

const lateQueue = new Queue("scoped");
const lateId = "late:1";
late();
