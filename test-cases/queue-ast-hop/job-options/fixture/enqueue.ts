import { createQueue, enqueueWidget } from "./factory";
import { PACKED } from "./names";
import * as defined from "./queues";
import {
  beforeComputed,
  computedKey,
  drafts,
  foreign,
  generated,
  hollow,
  hostQueue,
  items,
  labeled,
  nested,
  orders,
  packedViaConst,
  shadow,
  spreadQueue,
  widgets,
} from "./queues";

declare const mystery: object;
declare const dynamic: string;
declare const unresolvedName: string;
declare const parts: [string, object, object];
declare const rest: [object];
declare const bulkRest: object[];
declare function makeOpts(): object;
declare function extra(): object;
declare function makeItem(): object;
declare const flag: boolean;
declare const mysteryBulk: object[];
declare const unknownQueue: { add: (name: string, data: object) => void };

widgets.add("plain", {});
widgets.add("ready", {}, { backoff: 1 });
const partial = { attempts: 1, removeOnComplete: true };
widgets.add("partial", {}, { ...partial });
const alias = partial;
widgets.add("alias", {}, alias);
export function later() {
  widgets.add("later", {}, namedLater);
}
const namedLater = { backoff: 1 };
const loopA = { attempts: 1, ...loopB };
const loopB = { removeOnComplete: true, ...loopA };
widgets.add("cycle", {}, loopA);
const bad = { ...mystery };
widgets.add("bad", {}, bad);
widgets.add("open", {}, { attempts: 1, ...mystery });
widgets.add("bare", {}, mystery);
widgets.add("called", {}, makeOpts());
widgets.add("ternary", {}, flag ? { backoff: 1 } : { backoff: 1 });
widgets.add("paren", {}, ({ backoff: 1 }));
widgets.add("inline", {}, { ...{ backoff: 1 } });
widgets.add("spread-call", {}, { backoff: 1, ...extra() });
widgets.add("computed", {}, { [dynamic]: true, backoff: 1 });
widgets.add(...parts);
let ignored = { attempts: 1 };
widgets.add("mutable", {}, ignored);

widgets.addBulk([
  { name: "tight", data: {}, opts: { backoff: 1 } },
  { name: "loose", data: {} },
  ...bulkRest,
  makeItem(),
]);
widgets.addBulk(mysteryBulk);

items.add("pick", {});
items.add("pack", {}, { priority: 2 });
orders.add("ship", {});
orders.add("open-order", {}, mystery);
drafts.add("lose", {});
drafts.add("keep", {}, { attempts: 1, removeOnComplete: true, backoff: 1 });
beforeComputed.add("early", {});
computedKey.add("ck", {});
nested.add("n", {});
hollow.add("h", {});
shadow.add("dim", {});
shadow.add("shade", {}, mystery);
packedViaConst.add("via", {});
hostQueue.add("host", {});
foreign.add("abroad", {});
generated.add("gen", {});
spreadQueue.add("spr", {});
labeled.add("tag", {});

export const packed = createQueue(PACKED, {
  defaultJobOptions: { attempts: 1, removeOnComplete: true, backoff: 1 },
});
export const memory = createQueue("memory", "local");
export const lost = createQueue(unresolvedName, {
  defaultJobOptions: { attempts: 1 },
});
enqueueWidget(packed, "wrapped", {});
enqueueWidget(widgets, "wrapped-plain");
enqueueWidget(widgets, "spread-opts", ...rest);
enqueueWidget(defined.widgets, "namespaced", { backoff: 1 });
memory.add("m", {});
unknownQueue.add("ghost", {});

const inner = { attempts: 1 };
const left = { ...inner, removeOnComplete: true };
const right = { ...inner, backoff: 1 };
widgets.add("left", {}, left);
widgets.add("right", {}, right);

widgets.add("kept", {});
// no-mistakes-disable-next-line queues-check
widgets.add("skipped", {});
