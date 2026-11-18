import check from "@example/assertions";
import type { SessionExecutor } from "@example/db";
import { assertFound } from "./helpers";
import { byId, type Input } from "./query";

declare function ensure(value: unknown, status?: number, message?: string): asserts value;
declare function mutate(notice: Input): void;

export async function s1(query: SessionExecutor, notice: Input | undefined) {
  ensure(notice, 404, "Notice required");
  return byId(query, notice.id);
}

export async function s2(query: SessionExecutor, notice: Input | undefined) {
  assertFound(notice, 404, "Notice required");
  return byId(query, notice.id);
}

export async function s3(query: SessionExecutor, notice: Input) {
  check(notice, 404);
  return byId(query, notice.id);
}

export async function c1(query: SessionExecutor, notice: Input | undefined) {
  ensure(notice, 404, "Notice required");
  mutate(notice);
  return byId(query, notice.id);
}

export async function c2(query: SessionExecutor, notice: Input | undefined) {
  ensure(notice, 404, "Notice required");
  notice.id = "changed";
  return byId(query, notice.id);
}
