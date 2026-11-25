import { FlowProducer, Queue } from "bullmq";
import { makeEnqueue } from "./factory";

const bare = new Queue("bare");
bare.add("known-missing", {});

const enqueue = makeEnqueue({ name: "welcome" });
enqueue({});
enqueue({}, { removeOnFail: false });

export const trackedMirror = new Queue("tracked", { defaultJobOptions: { removeOnFail: true } });
const flow = new FlowProducer();
flow.add({ name: "audit", queueName: "tracked", data: {} });
