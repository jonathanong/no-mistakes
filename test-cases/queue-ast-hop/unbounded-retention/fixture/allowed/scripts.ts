import { Queue } from "bullmq";
import { ALLOWED_QUEUE_NAME, bare } from "../queues";

declare const dynKey: string;
declare const extraOpts: Record<string, unknown>;
declare const UNKNOWN_QUEUE_NAME: string;
declare function getRetention(): boolean | number;

const importedAllowed = new Queue(ALLOWED_QUEUE_NAME, {
  defaultJobOptions: { removeOnComplete: true, removeOnFail: true },
});
const unknownDefaults = new Queue(UNKNOWN_QUEUE_NAME, {
  defaultJobOptions: { removeOnComplete: true },
});
void unknownDefaults;
importedAllowed.add(
  "via-imported-const",
  {},
  { attempts: 3, removeOnComplete: true, removeOnFail: true },
);

const keepBounded = { count: 50 };
const keepEmpty = {};
let mutKeep = { count: 10 };
const loopA: Record<string, unknown> = loopB;
const loopB: Record<string, unknown> = loopA;
const openOpts = { ...extraOpts };

// Exempted by queues.unboundedJobRetention.allowPaths.
bare.add("allowed-script", {}, { removeOnComplete: true, removeOnFail: false });
bare.add("allowed-empty", {}, { removeOnComplete: {}, removeOnFail: keepEmpty });
bare.add("allowed-unknown", {}, { removeOnComplete: getRetention(), removeOnFail: -1 });
bare.add(
  "allowed-keep-variants",
  {},
  {
    removeOnComplete: { count: null, [dynKey]: 1 },
    removeOnFail: { age: undefined, extra: 1, ...keepBounded, ...extraOpts },
  },
);
bare.add(
  "allowed-keep-spread-edge",
  {},
  {
    removeOnComplete: { ...keepEmpty, ...{ count: 10 }, ...mutKeep },
    removeOnFail: { ...(getRetention() as object), ...loopA },
  },
);
bare.add("allowed-open-const", {}, { removeOnComplete: true, ...openOpts });
