import { Queue, Worker } from "bullmq";

const queue = new Queue("scope", { defaultJobOptions: { removeOnComplete: true, backoff: 1 } });

export function completeFirst() {
  const opts = { attempts: 3 };
  queue.add("complete-first", {}, opts);
}

export function incompleteSecond() {
  const opts = {};
  queue.add("incomplete-second", {}, { ...opts });
}

export function incompleteFirst() {
  const options = {};
  queue.add("incomplete-first", {}, options);
}

export function completeSecond() {
  const options = { attempts: 3 };
  queue.add("complete-second", {}, { ...options });
}

declare function loadOptions(): object;
const opts = {};
export function opaqueShadow() {
  const opts = loadOptions();
  queue.add("opaque-shadow", {}, opts);
}
export function parameterShadow(opts: object) {
  queue.add("parameter-shadow", {}, { ...opts });
}
queue.add("outer-opts", {}, opts);
new Worker("scope", async () => {});
