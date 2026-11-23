import type { NoSingleEnqueueInLoopOptions, NoSingleEnqueueInLoopTarget } from "no-mistakes";

const target: NoSingleEnqueueInLoopTarget = { module: "./queue/widgets", export: "enqueueWidget" };
const explicit: NoSingleEnqueueInLoopOptions = { single: [target], bulk: [] };
const pattern: NoSingleEnqueueInLoopOptions = {
  singleExportPattern: "^enqueue[A-Z]",
  bulkExportPattern: "^enqueueBulk",
  moduleGlobs: ["./queue/*"],
};
const empty: NoSingleEnqueueInLoopOptions = {};
// @ts-expect-error Pattern values are serialized regex text.
const regexObject: NoSingleEnqueueInLoopOptions = { singleExportPattern: /^enqueue/ };
// @ts-expect-error Module globs are strings.
const invalidGlob: NoSingleEnqueueInLoopOptions = { moduleGlobs: [1] };
// @ts-expect-error Explicit export names are strings.
const invalidTarget: NoSingleEnqueueInLoopTarget = { module: "./queue/widgets", export: false };
void [explicit, pattern, empty, regexObject, invalidGlob, invalidTarget];
