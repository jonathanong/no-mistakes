import { Queue } from "bullmq";

declare const mystery: object;
declare const dynamic: string;
declare function makeOptions(): object;
declare const queueOpts: object[];

export const widgets = new Queue("widgets", {
  ...mystery,
  defaultJobOptions: widgetDefaults,
});
const widgetDefaults = { attempts: 1, removeOnComplete: true };
const { backoff: renamed } = widgetDefaults;

export const items = new Queue("items");
export const orders = new Queue("orders", {
  defaultJobOptions: { attempts: 1, ...mystery },
});
export const drafts = new Queue("drafts", {
  defaultJobOptions: { attempts: 1, removeOnComplete: true, backoff: 1 },
  ...mystery,
});
export const beforeComputed = new Queue("beforeComputed", {
  [dynamic]: true,
  defaultJobOptions: { attempts: 1, removeOnComplete: true },
});
export const computedKey = new Queue("computedKey", {
  defaultJobOptions: { attempts: 1, removeOnComplete: true, backoff: 1 },
  [dynamic]: true,
});
export const nested = new Queue("nested", {
  ...{ defaultJobOptions: { backoff: 1 } },
});
export const hollow = new Queue("hollow", {
  ...{ connection: "local" },
});
export const shadow = new Queue("shadow", { ...mystery });
export const packedOptions = {
  defaultJobOptions: { attempts: 1, removeOnComplete: true, backoff: 1 },
};
export const packedViaConst = new Queue("packedViaConst", packedOptions);
export const hostOnly = { host: "local" };
export const hostQueue = new Queue("hostQueue", hostOnly);
declare const foreignOptions: object;
export const foreign = new Queue("foreign", foreignOptions);
export const generated = new Queue("generated", makeOptions());
export const spreadQueue = new Queue("spreadQueue", ...queueOpts);
export const labeled = new Queue("labeled", { connection: "local" });
