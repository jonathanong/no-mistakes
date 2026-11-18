import { read } from "@example/db";
import { requireValue } from "./helpers";
import { byId, type Input } from "./query";

export function afterEval(input: Input) {
  eval("requireValue = replacement");
  requireValue(input);
  return byId(read, input.id);
}
