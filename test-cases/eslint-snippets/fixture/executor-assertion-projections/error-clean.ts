import { read } from "@example/db";
import { requireEnabled } from "./helper-error";
import { byId, type Input } from "./query";

export function afterAssertion(input: Input) {
  try {
    requireEnabled(input, false);
  } catch {}
  return byId(read, input.id);
}
