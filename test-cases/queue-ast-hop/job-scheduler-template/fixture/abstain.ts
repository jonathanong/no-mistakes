import { Queue } from "bullmq";

const orders = new Queue("orders");
const shared = { delay: 1 };
const tdzOpts = { delay: 9 };
const delayOpts = { delay: 8 };
const label = { delay: 1 };
const nested = { jobId: "outer" };
const marker = { parent: { id: "outer" } };
const caught = { jobId: "outer" };
const item = { delay: 1 };
const pair = { parent: { id: "outer" } };
const fallback = { jobId: "outer" };
const itemRest = { deduplication: { id: "outer" } };
const a = b;
const b = a;
let mutable = { delay: 1 };
const sequence = (0, { jobId: "seq" });
const { skipped } = { delay: 1 };
const [first, ...restNames] = [0];
const { alias = loadOptions() } = {};
void first;
void restNames;
void skipped;
void alias;

declare function loadOptions(): object;

orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  opts: { delay: 5, ...loadOptions() },
});

orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  delay: 1,
  ...extra,
});

const key = "delay";
orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  opts: { [key]: 1, jobId: "hidden" },
});

orders.upsertJobScheduler("daily", { every: 1000 }, a);
orders.upsertJobScheduler("daily", { every: 1000 }, mutable);
orders.upsertJobScheduler("daily", { every: 1000 }, sequence);
orders.upsertJobScheduler("daily", { every: 1000 }, loadOptions());
orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  opts: loadOptions(),
});

function scheduleShadow(shared: object) {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: shared,
  });
}

function scheduleTdz() {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: tdzOpts,
  });
  const tdzOpts = { attempts: 1 };
}

const scheduleArrow = (delayOpts: object) => {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: delayOpts,
  });
};

const scheduleNamed = function label() {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: label,
  });
};

function scheduleNested() {
  function nested() {
    return 0;
  }
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: nested,
  });
  return nested;
}

function scheduleClass() {
  class marker {}
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: marker,
  });
  return marker;
}

function scheduleCatch() {
  try {
    throw new Error("skip");
  } catch (caught) {
    orders.upsertJobScheduler("daily", { every: 1000 }, {
      name: "pack",
      opts: caught,
    });
  }
}

function scheduleRest(...items: object[]) {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: items[0],
  });
}

function scheduleDestructured({ item }: { item: object }) {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: item,
  });
}

function scheduleArray([, pair, ...restItems]: [number, object, ...object[]]) {
  void restItems;
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: pair,
  });
}

function scheduleDefault(fallback = loadOptions()) {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: fallback,
  });
}

function scheduleRestProps({ ...itemRest }: { delay?: number }) {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: itemRest,
  });
}

void scheduleShadow;
void scheduleTdz;
void scheduleArrow;
void scheduleNamed;
void scheduleNested;
void scheduleClass;
void scheduleCatch;
void scheduleRest;
void scheduleDestructured;
void scheduleArray;
void scheduleDefault;
void scheduleRestProps;
