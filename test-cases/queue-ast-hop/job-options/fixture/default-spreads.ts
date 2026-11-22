import { Queue } from "bullmq";
import { createQueue, foreignDefaults } from "./factory";

declare const unknown: object;
const complete = {
  defaultJobOptions: { attempts: 1, removeOnComplete: true, backoff: 1 },
};
const partial = { defaultJobOptions: { attempts: 1, removeOnComplete: true } };
const aliased = partial;
const connection = { connection: "local" };
const composed = { ...unknown, ...complete, ...connection };

const known = new Queue("known", { ...composed, ...complete, ...connection });
known.add("defaults-from-known-spread", {});

const replaced = new Queue("replaced", { ...complete, ...aliased, ...connection });
replaced.add("defaults-after-known-override", {});

const open = new Queue("openDefaults", { ...complete, ...unknown });
open.add("defaults-after-unknown", {});

const fixed = new Queue("fixed", { ...unknown, ...complete });
fixed.add("defaults-after-known", {});

const factoryQueue = createQueue("factoryQueue", { ...aliased });
factoryQueue.add("defaults-from-factory-spread", {});

function withShadow(partial: object) {
  const shadowed = new Queue("shadowed", { ...partial });
  shadowed.add("defaults-from-shadow", {});
}

const cycleA = { ...cycleB };
const cycleB = { ...cycleA };
const circular = new Queue("circular", { ...cycleA });
circular.add("defaults-from-cycle", {});

let mutableOptions = partial;
mutableOptions = complete;
const mutableQueue = createQueue("mutableQueue", { ...mutableOptions });
mutableQueue.add("defaults-from-mutable", {});

const importedQueue = createQueue("importedQueue", { ...foreignDefaults });
importedQueue.add("defaults-from-import", {});
