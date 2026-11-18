import ensure from "@example/guard-overloads";
import { read } from "@example/db";
import { visible } from "./overload-helpers";
import { byId, type Input } from "./query";

export function ambientOverloads(input: Input | undefined) {
  ensure(input, 404, "Missing input");
  return byId(read, input.id);
}

export function ambientShortOverload(input: Input | undefined) {
  ensure(input);
  return byId(read, input.id);
}

export function visibleOverloads(input: Input | undefined) {
  visible(input, 404);
  return byId(read, input.id);
}

export function ambientOptionalMessage(input: Input | undefined) {
  ensure(input, 404);
  return byId(read, input.id);
}
