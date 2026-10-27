import { enqueueBatch as enqueueBatchImpl } from "@app/jobs";
import { submitForKey } from "./enqueue";
import { dispatchDropped, dispatchTagged, dispatchInterpolated, dispatchPlainInterpolated } from "./dispatch";

// The test callback keeps the imported job's exact type through every forwarding hop.
const enqueueAndRecord: typeof enqueueBatchImpl = (entries, options) => {
  void entries;
  return enqueueBatchImpl(entries, options);
};
await submitForKey("item", { enqueueBatch: enqueueAndRecord, computeDelay: async () => 1, now: new Date(0) });

const dropped: typeof enqueueBatchImpl = (entries, options) => {
  return enqueueBatchImpl(entries, options);
};
await dispatchDropped({ enqueueBatch: dropped, computeDelay: async () => 1 });

const tagged: typeof enqueueBatchImpl = (entries, options) => {
  return enqueueBatchImpl(entries, options);
};
await dispatchTagged({ enqueueBatch: tagged, computeDelay: async () => 1 });

const interpolated: typeof enqueueBatchImpl = (entries, options) => {
  return enqueueBatchImpl(entries, options);
};
await dispatchInterpolated({ enqueueBatch: interpolated, computeDelay: async () => 1 });

const plainInterpolated: typeof enqueueBatchImpl = (entries, options) => {
  return enqueueBatchImpl(entries, options);
};
await dispatchPlainInterpolated({ enqueueBatch: plainInterpolated, computeDelay: async () => 1 });

// Resolving this imported promise alias must retain the indirect dropped call.
export function invokeDropped(callback: typeof enqueueBatchImpl) {
  callback([], {});
}
