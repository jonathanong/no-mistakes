import type check from "@example/assertions";
import { type namedCheck } from "@example/assertions";
import { read } from "@example/db";
import { byId, type Input } from "./query";

export function afterTypeOnlyDefault(input: Input) {
  check(input);
  return byId(read, input.id);
}

export function afterTypeOnlyNamed(input: Input) {
  namedCheck(input);
  return byId(read, input.id);
}
