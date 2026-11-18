import { read } from "@example/db";
import { checkArrow, checkFunction } from "@example/arrow-assertions";
import { byId, type Input } from "./query";

export function throughArrow(input: Input) {
  checkArrow(input);
  return byId(read, input.id);
}

export function throughFunction(input: Input) {
  checkFunction(input);
  return byId(read, input.id);
}
