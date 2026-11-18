import ensure from "@example/guard-overloads";
import * as guards from "./helpers";
import { read } from "@example/db";
import { byId, type Input } from "./query";

ensure.equal = () => {};

export function afterMergedNamespaceWrite(input: Input) {
  ensure(input, 404, "Missing input");
  return byId(read, input.id);
}

guards.requireValue = guards.writeInAssertion;

export function afterNamedNamespaceWrite(input: Input) {
  guards.requireValue(input);
  return byId(read, input.id);
}
