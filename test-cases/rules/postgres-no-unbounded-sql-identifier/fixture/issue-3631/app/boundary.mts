import { read } from "@example/db";
import { KINDS } from "./dynamic-barrel.mts";
import type { Kind } from "./dynamic-barrel.mts";
import { OUTSIDE } from "../outside/kinds.mts";
export function dynamicAlias(k: Kind) {
  return read(`SELECT * FROM ${k}`);
}
export function dynamicTuple(k: (typeof KINDS)[number]) {
  return read(`SELECT * FROM ${k}`);
}
export function outside(k: (typeof OUTSIDE)[number]) {
  return read(`SELECT * FROM ${k}`);
}
import type { Looped } from "./cycle-a.mts";
import type { Kind as StarKind } from "./star-type.mts";
export function cycle(k: Looped) {
  return read(`SELECT * FROM ${k}`);
}
export function typeStar(k: StarKind) {
  return read(`SELECT * FROM ${k}`);
}
