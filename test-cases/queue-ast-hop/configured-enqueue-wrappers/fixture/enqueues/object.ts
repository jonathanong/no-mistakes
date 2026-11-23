import { objects } from "../queues";
import { makeObjectEnqueue, makeObjectProducer } from "../lib/wrappers";

const sendObject = makeObjectEnqueue({
  queue: objects,
  queueName: "objects",
  jobName: "objectTask",
  defaults: { attempts: 3, removeOnComplete: true },
});

sendObject({}, {
  priority: 0,
  jobId: "bad:object",
  deduplication: { mode: "debounce", ttl: 100 },
});
sendObject({}, { priority: 3 });

makeObjectProducer({
  queueName: "objects",
  jobName: "directTask",
  defaults: { attempts: 3, removeOnComplete: true, priority: 3 },
});

const noDefaults = makeObjectEnqueue({
  queue: objects,
  queueName: "objects",
  jobName: "noDefaultsTask",
  defaults: {},
});
noDefaults({}, { removeOnComplete: true, priority: 3 });
