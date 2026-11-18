import type { SessionExecutor } from "@example/db";
import { check } from "./fresh-error-helper";
import { byId, type Input } from "./query";

Object.assign = ((value: unknown) => value) as typeof Object.assign;

export async function afterAssignWrite(query: SessionExecutor, notice: Input | undefined) {
  check(notice, 404);
  return byId(query, notice.id);
}
