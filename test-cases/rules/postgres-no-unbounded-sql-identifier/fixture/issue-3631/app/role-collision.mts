import { read } from "@example/db";
import type { Shape } from "./role-provider.mts";
export function open(k: Shape) {
  return read(`SELECT * FROM ${k}`);
}
export function tuple(k: (typeof Shape)[number]) {
  return read(`SELECT * FROM ${k}`);
}
open("outside");
