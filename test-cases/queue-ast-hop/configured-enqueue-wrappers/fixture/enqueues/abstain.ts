import { MUTABLE_DEF, SPREAD_DEF, UNRESOLVED_DEF } from "../definitions";
import { returned } from "../queues";
import { makeFixedEnqueue, makeReturnedEnqueue } from "../lib/wrappers";

declare function unconfiguredLookalike(
  definition: { name: string },
): (data: unknown, options: {
  attempts: number;
  removeOnComplete: boolean;
  priority: number;
}) => void;

const unresolved = makeFixedEnqueue(UNRESOLVED_DEF);
unresolved({}, { attempts: 2, removeOnComplete: true, priority: 3 });

const spread = makeFixedEnqueue(SPREAD_DEF);
spread({}, { attempts: 2, removeOnComplete: true, priority: 3 });

const mutable = makeFixedEnqueue(MUTABLE_DEF);
mutable({}, { attempts: 2, removeOnComplete: true, priority: 3 });

let localDefinition = { name: "localMutableTask" };
const local = makeFixedEnqueue(localDefinition);
local({}, { attempts: 2, removeOnComplete: true, priority: 3 });

let returnedCallable = makeReturnedEnqueue(returned, "mutableCallableTask");
returnedCallable({}, { attempts: 2, removeOnComplete: true, priority: 3 });

const lookalike = unconfiguredLookalike({ name: "unconfiguredTask" });
lookalike({}, { attempts: 2, removeOnComplete: true, priority: 3 });
