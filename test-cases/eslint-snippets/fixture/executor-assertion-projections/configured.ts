import check, { namedCheck as verify } from "@example/assertions";
import { read } from "@example/db";
import { byId, type Input } from "./query";

export function throughConfiguredDefault(input: Input) {
  check(input);
  return byId(read, input.id);
}

export function throughConfiguredNamed(input: Input) {
  verify(input);
  return byId(read, input.id);
}
