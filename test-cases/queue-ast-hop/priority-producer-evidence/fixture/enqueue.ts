import { Queue, Worker } from "bullmq";
import {
  ambiguousFlow,
  computed,
  crossFileFlow,
  duplicate,
  numeric,
  sameCase,
  sameLine,
  shadowed,
  staticBlock,
  unknownBulkName,
  unknownComputed,
} from "./queues";
import * as queueModule from "./queues";

declare const key: string;
declare const route: number;
declare const fields: Record<string, unknown>;
declare const jobs: { name: string; data: object }[];

sameLine.add("plain", {}); sameLine.add("plain", {});
sameLine.add("rush1", {}, { priority: 1 });
sameLine.add("rush2", {}, { priority: 1 });
sameLine.add("rush3", {}, { priority: 1 });

const LARGE = 4_294_967_296;
numeric.add("plain", {});
numeric.add("large", {}, { priority: LARGE });

computed.add("plain", {});
computed.add("rush", {}, { ["priority"]: 5 });
unknownComputed.add("plain", {});
unknownComputed.add("unknown", {}, { priority: 5, [key]: 0 });

duplicate.addBulk([{ name: "oldName", name: "newName", data: {} }]);
duplicate.add("rush1", {}, { priority: 1 });
duplicate.add("rush2", {}, { priority: 1 });
unknownBulkName.addBulk([{ name: "uncertain", ...fields, data: {} }]);
unknownBulkName.add("rush", {}, { priority: 5 });

export function callOtherQueue(shadowed: Queue, queueModule: { shadowed: Queue }) {
  shadowed.addBulk(jobs);
  queueModule.shadowed.addBulk(jobs);
}
shadowed.add("plain", {});
shadowed.add("rush", {}, { priority: 5 });
shadowed.add("anotherRush", {}, { priority: 5 });

const LEVEL = 0;
export class ClassWithStaticBlock {
  static {
    const LEVEL = 6;
    staticBlock.add("duringStatic", {}, { priority: LEVEL });
  }

  enqueue() {
    staticBlock.add("classMethod", {}, { priority: LEVEL });
  }
}

switch (route) {
  case 0: {
    break;
  }
  case 1:
    const submit = () => sameCase.add("caseRush", {}, { priority: CASE_LEVEL });
    const CASE_LEVEL = 7;
    submit();
}
sameCase.add("plain", {});

crossFileFlow.add("direct", {});
ambiguousFlow.add("direct", {});

const localQueue = new Queue("localQueue");
new Worker("localQueue", async () => {});
export function shadowLocalQueue(localQueue: Queue) {
  localQueue.addBulk(jobs);
}
localQueue.add("plain", {});
localQueue.add("rush", {}, { priority: 5 });
