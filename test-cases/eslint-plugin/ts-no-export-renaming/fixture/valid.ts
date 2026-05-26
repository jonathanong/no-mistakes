import { X } from "x";

const local = 1;
type Alias = string;
type Count = number;

export { X };
export { local };
export type { Alias as PublicAlias };
export { type Count as PublicCount };
export * as namespace from "x";
