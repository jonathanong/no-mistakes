import type { SessionExecutor } from "@example/db";
import { check } from "./fresh-error-helper-write";
import { byId, type Input } from "./query";

export async function afterHelperWrite(query: SessionExecutor, notice: Input | undefined) {
  check(notice, 404);
  return byId(query, notice.id);
}
