export const KINDS = ["a", "b", "c"] as const;
export type Kind = (typeof KINDS)[number];
type Chained = Kind;
export type { Chained as RenamedKind };
const HIDDEN = ["x", "y"] as const;
export { HIDDEN as RENAMED };
