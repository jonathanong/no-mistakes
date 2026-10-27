import { enqueueBatch } from "@app/jobs";
import { submitForKey } from "./enqueue.js";
import { dispatchDropped, dispatchTagged, dispatchInterpolated, dispatchPlainInterpolated } from "./dispatch.js";

// Keep all forwarding modules: a direct handler call hides the TypeScript-only gap.
const enqueueAndRecord = (entries, options) => {
  void entries;
  return enqueueBatch(entries, options);
};
await submitForKey("item", { enqueueBatch: enqueueAndRecord, computeDelay: async () => 1, now: new Date(0) });

const dropped = (entries, options) => {
  return enqueueBatch(entries, options);
};
await dispatchDropped({ enqueueBatch: dropped, computeDelay: async () => 1 });

const tagged = (entries, options) => {
  return enqueueBatch(entries, options);
};
await dispatchTagged({ enqueueBatch: tagged, computeDelay: async () => 1 });

const interpolated = (entries, options) => {
  return enqueueBatch(entries, options);
};
await dispatchInterpolated({ enqueueBatch: interpolated, computeDelay: async () => 1 });

const plainInterpolated = (entries, options) => {
  return enqueueBatch(entries, options);
};
await dispatchPlainInterpolated({ enqueueBatch: plainInterpolated, computeDelay: async () => 1 });
