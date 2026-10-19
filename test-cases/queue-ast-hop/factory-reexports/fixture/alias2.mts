// `export { x as y } from` is not a local binding. Type specifiers are not values.
export { type QN as OnlyType, bg as viaExportFrom } from "./q.mts";
export type { JOB_IMPORTED as JobType } from "./names.mts";
// A missing module is unresolved, not a queue export.
export { bg as missing } from "./missing.mts";
