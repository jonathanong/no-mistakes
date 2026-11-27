export type Missing = never;
type LocalMissing = never;
export type { LocalMissing as PublicMissing };
// An imported type's definition cannot be judged from this file.
import type { RemoteMissing } from "./types";
export type { RemoteMissing };
export type { MissingElsewhere } from "./types";
export type * from "./types";
// no-mistakes-disable-next-line no-placeholder-never-type-exports -- definition is outside this file
export type { SuppressedRemote } from "@example/types";
export type { VisibleRemote } from "@example/types";
const value = 1;
type LocalValue = string;
export { type LocalValue, value };
export { type LocalMissing, value as renamedValue };
export type Visible = string;
export type { Visible as PublicVisible };
export interface Item {}
export type { Item as PublicItem };
class Service {}
export type { Service };
enum Kind { Example }
export type { Kind };
declare namespace Group {}
export type { Group };
export type { Missing as PublicMissingAlias };
