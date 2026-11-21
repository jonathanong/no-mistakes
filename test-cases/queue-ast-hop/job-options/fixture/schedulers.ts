import { items, widgets } from "./queues";

declare const mystery: object;
declare const templateRest: object[];
declare const scheduleParts: [string, object, object];
declare function makeTemplate(): object;

widgets.upsertJobScheduler("hourly", { every: 60000 }, { name: "sweep", data: {}, opts: {} });
widgets.upsertJobScheduler(
  "kept",
  { every: 60000 },
  { name: "sweep", data: {}, opts: { backoff: 1 } },
);
widgets.upsertJobScheduler("mystery", { every: 60000 }, mystery);
items.upsertJobScheduler("pick", { every: 60000 }, { name: "pick", data: {} });
const sweepTemplate = { name: "sweep", data: {}, opts: { backoff: 1 } };
widgets.upsertJobScheduler("from-const", { every: 1000 }, sweepTemplate);
const bareTemplate = { name: "bare", data: {} };
widgets.upsertJobScheduler("bare-template", { every: 1000 }, bareTemplate);
widgets.upsertJobScheduler("no-template", { every: 1000 });
widgets.upsertJobScheduler("spread-template", { every: 1000 }, ...templateRest);
widgets.upsertJobScheduler(...scheduleParts);
widgets.upsertJobScheduler("called", { every: 1000 }, makeTemplate());
