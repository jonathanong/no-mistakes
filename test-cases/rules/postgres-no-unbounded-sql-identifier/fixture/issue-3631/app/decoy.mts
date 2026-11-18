export const KINDS = ["a", "b"] as const;
export type Kind = (typeof KINDS)[number];
export const unrelated = 1;
