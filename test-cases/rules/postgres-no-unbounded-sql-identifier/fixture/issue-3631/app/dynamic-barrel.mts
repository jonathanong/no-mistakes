export const KINDS = (await import("./kinds.mts")).KINDS;
export type Kind = (typeof KINDS)[number];
