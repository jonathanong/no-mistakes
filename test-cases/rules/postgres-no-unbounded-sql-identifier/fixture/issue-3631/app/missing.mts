import { read } from "@example/db";
import type { Kind } from "./absent.mts";
import { KINDS } from "./absent.mts";
import { unrelated } from "./decoy.mts";
export function missingAlias(k: Kind) {
  return read(`SELECT * FROM ${k}`);
}
export function missingTuple(k: (typeof KINDS)[number]) {
  return read(`SELECT * FROM ${k}`);
}
