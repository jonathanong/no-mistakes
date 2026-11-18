import { namedCheck } from "@example/assertions";
import { read } from "@example/db";
import { byId, type Input } from "./query";

export function beforeProjection(input: Input) {
  namedCheck(input);
  return byId(read, input.id);
}
