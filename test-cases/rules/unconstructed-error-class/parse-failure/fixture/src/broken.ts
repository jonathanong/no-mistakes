// Does not parse, and holds the only `new Used()`. The rule cannot prove
// `Used` unconstructed without reading this file, so it must stop instead.
import { Used } from "./used";

export function make() {
  return new Used();
}

const unterminated = {;
