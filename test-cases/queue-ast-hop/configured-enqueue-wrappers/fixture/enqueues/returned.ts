import { returned } from "../queues";
import { makeReturnedEnqueue } from "../lib/wrappers";

const send = makeReturnedEnqueue(returned, "returnedTask");
const alias = send;

alias({}, {
  attempts: 2,
  removeOnComplete: true,
  priority: 0,
  jobId: "bad:returned",
  deduplication: { mode: "debounce", ttl: 100 },
});
send({}, { removeOnComplete: true, priority: 3 });
