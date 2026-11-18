import type { SessionExecutor } from "@example/db";
import { check, explicitMetadata, emptyMetadata } from "./fresh-error-helper";
import { byId, type Input } from "./query";

export async function load(query: SessionExecutor, notice: Input | undefined) {
  check(notice, 404);
  return byId(query, notice.id);
}

export async function withExplicitMetadata(query: SessionExecutor, notice: Input | undefined) {
  explicitMetadata(notice, 404);
  return byId(query, notice.id);
}

export async function withEmptyMetadata(query: SessionExecutor, notice: Input | undefined) {
  emptyMetadata(notice, 404);
  return byId(query, notice.id);
}
