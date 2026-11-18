import type { SessionExecutor } from "@example/db";
import { check } from "./fresh-error-helper";
import { byId, type Input } from "./query";

const prototype = Error.prototype;
Object.defineProperty(prototype, "status", {
  set(value: number) {},
});

export async function afterAliasedPrototypeWrite(
  query: SessionExecutor,
  notice: Input | undefined,
) {
  check(notice, 404);
  return byId(query, notice.id);
}
