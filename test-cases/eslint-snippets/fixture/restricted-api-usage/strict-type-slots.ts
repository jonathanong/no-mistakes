declare const api: any;
declare const queue: any[];
declare let blockingWait: any;
declare let order: any;

const read = blockingWait;
const member = api.blockingWait;
const incremented = blockingWait++;
const memberIncremented = api.blockingWait++;
order = blockingWait;
for (order of blockingWait) {
}
for (order in blockingWait) {
}
blockingWait = 1;
api.blockingWait = 1;
blockingWait += 1;
api.blockingWait += 1;
for (blockingWait of queue) {
}
for (blockingWait in queue) {
}
for (api.blockingWait of queue) {
}
for (api.blockingWait in queue) {
}

namespace widgets {
  type blockingWait = { item: number };
  type box = { blockingWait: number };
  const item: blockingWait = 0;
  type nested = blockingWait.Item;
  type imported = import("widgets").blockingWait;
  const queried: typeof blockingWait = blockingWait;
  function boxOf<blockingWait>(item: blockingWait): blockingWait {
    return item;
  }
}

namespace items {
  interface blockingWait {
    blockingWait: number;
  }
  interface reader {
    blockingWait(): void;
  }
  interface widget extends blockingWait {
    [blockingWait]: number;
  }
  interface shipment extends api.blockingWait {
    item: number;
  }
  class order implements blockingWait {}
}

namespace orders {
  enum blockingWait {
    blockingWait = 1,
    item = blockingWait,
  }
}

namespace shipments {
  namespace blockingWait {
    const item = 1;
  }
}

function isOrder(blockingWait: unknown): blockingWait is string {
  return false;
}
