import { read } from "@example/db";
import * as assertions from "./helpers";
import { requireValue } from "./helpers";
import { byId, type Input } from "./query";

assertions.requireValue = assertions.writeInAssertion;

export function afterNamespaceWrite(input: Input) {
  assertions.requireValue(input);
  return byId(read, input.id);
}

export function afterNamespaceWriteThroughNamedImport(input: Input) {
  requireValue(input);
  return byId(read, input.id);
}
