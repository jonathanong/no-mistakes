import { Queue } from "bullmq";

const orders = new Queue("orders");

const packOptions = { attempts: 2, jobId: "pack-1" };

function scheduleEarly() {
  orders.upsertJobScheduler("daily", { every: 1000 }, earlyTemplate);
}

const earlyTemplate = { name: "pack", opts: { delay: 2 } };
const row = { parent: { id: "outer" } };
const item = { deduplication: { id: "outer" } };
const bucket = { delay: 12 };
const parenthesized = { delay: 8 };

export const exportedTemplate = {
  name: "pack",
  opts: { ["parent"]: { id: "root" } },
};

const widgetTemplate = {
  name: "pack",
  opts: { [`jobId`]: "widget" },
} as const;

class WidgetBatch {
  schedule() {
    orders.upsertJobScheduler("daily", { every: 1000 }, {
      name: "pack",
      delay: 4,
      opts: { delay: 5 },
    });
  }
}

function scheduleRows(items: object[]) {
  for (const row of items) {
    orders.upsertJobScheduler("daily", { every: 1000 }, {
      name: "pack",
      opts: row,
    });
  }
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: row,
  });
}

function scheduleItem() {
  {
    const item = loadItem();
    orders.upsertJobScheduler("daily", { every: 1000 }, {
      name: "pack",
      opts: item,
    });
  }
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: item,
  });
}

declare function loadItem(): object;

orders.upsertJobScheduler("daily", { every: 1000 }, {
  name: "pack",
  data: { delay: 1 },
  opts: { ...packOptions, removeOnComplete: true },
});

orders.upsertJobScheduler("hourly", { every: 1000 }, {
  name: "pack",
  ...{ delay: 3 },
});

orders["upsertJobScheduler"]("hourly", { every: 1000 }, {
  name: "pack",
  opts: { deduplication: { id: "pack" } },
});

orders.upsertJobScheduler("weekly", { every: 1000 }, widgetTemplate);
orders.upsertJobScheduler("weekly", { every: 1000 }, (exportedTemplate));
orders.upsertJobScheduler("weekly", { every: 1000 }, {
  name: "pack",
  opts: (parenthesized),
});

function scheduleLocal() {
  const localOpts = { deduplication: { id: "local" } };
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: localOpts,
  });
}

const scheduleAnon = function () {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: { delay: 6 },
  });
};

const scheduleExpr = () =>
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: { delay: 7 },
  });

function scheduleIn(groups: Record<string, number>) {
  for (const bucket in groups) {
    orders.upsertJobScheduler("daily", { every: 1000 }, {
      name: "pack",
      opts: { attempts: 1 },
    });
  }
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: bucket,
  });
}

for (let step = 0; step < 1; step += 1) {
  orders.upsertJobScheduler("daily", { every: 1000 }, {
    name: "pack",
    opts: { delay: 11 },
  });
}

void scheduleAnon;
void scheduleExpr;
void scheduleIn;
