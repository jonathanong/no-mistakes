import { read } from "@example/db";
import * as assertions from "./helpers";
import { requireValue } from "./helpers";
import { byId, type Input } from "./query";

declare const key: string;
assertions[key] = assertions.writeInAssertion;

export function afterComputedNamespaceWrite(input: Input) {
  requireValue(input);
  return byId(read, input.id);
}
