import { read } from "@example/db";
import { requireValue } from "./type-only-star";
import { byId, type Input } from "./query";

export function afterTypeOnlyStar(input: Input) {
  requireValue(input);
  return byId(read, input.id);
}
