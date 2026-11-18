import type { SessionExecutor } from "@example/db";
import { check } from "./fresh-error-helper";
import { byId, type Input } from "./query";

Object.defineProperty(Error.prototype, "status", {
  set(value: number) {},
});

export async function afterPrototypeWrite(query: SessionExecutor, notice: Input | undefined) {
  check(notice, 404);
  return byId(query, notice.id);
}
