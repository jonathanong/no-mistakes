import { JOB_DEF, BULK_DEF } from "../definitions";
import { makeFixedEnqueue, makeFixedBulkEnqueue } from "../lib/wrappers";

const send = makeFixedEnqueue(JOB_DEF);
send({}, {
  attempts: 2,
  removeOnComplete: true,
  priority: 0,
  jobId: "bad:fixed",
  deduplication: { mode: "debounce", ttl: 100 },
});

const bulk = makeFixedBulkEnqueue(BULK_DEF, x => x);
bulk([], { attempts: 2, removeOnComplete: true, priority: 3 });
