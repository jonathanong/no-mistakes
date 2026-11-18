import check from "@example/assertions";
import { mutableCheck } from "@example/mutable-assertions";
import { read } from "@example/db";
import { byId, type Input } from "./query";

check = mutableCheck;

export function afterImportReassignment(input: Input) {
  check(input);
  return byId(read, input.id);
}

export function afterExportReassignment(input: Input) {
  mutableCheck(input);
  return byId(read, input.id);
}
