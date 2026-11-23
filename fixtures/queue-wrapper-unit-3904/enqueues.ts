import { DEFINITION, MUTABLE, SPREAD } from "./barrel";
import { createQueue, enqueuePositional, makeEnqueue } from "./wrappers";

const defaults = { attempts: 4, priority: 3, jobId: "bad:default" };
const send = makeEnqueue(DEFINITION, defaults);
const alias = send;

alias({}, { removeOnComplete: true, jobId: "safe" });
send({}, { removeOnComplete: true });
makeEnqueue({ name: "uninvoked" }, defaults);

function shadow(send: ReturnType<typeof makeEnqueue>) {
  send({});
}

let mutable = makeEnqueue(DEFINITION, defaults);
mutable({});
const unknownDefinition = makeEnqueue(MUTABLE, defaults);
unknownDefinition({});
const spreadDefinition = makeEnqueue(SPREAD, defaults);
spreadDefinition({});

const unknownOpts = makeEnqueue({ name: "unknown-options" }, defaults);
unknownOpts({}, ...unknownArgs);
declare const unknownArgs: [object];

const queue = createQueue("wrapper-legacy");
enqueuePositional(queue, "legacy", { attempts: 1, removeOnComplete: true });
