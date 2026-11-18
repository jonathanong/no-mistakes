import { read, write } from "@example/db";
import { afterDestructuredExecutor, byId, canEdit, withInput } from "./helpers";
import type { Input } from "./local";

interface Row {
  id: string;
}

export function beforeImportedScalarHelper(input: Input) {
  canEdit(input);
  return byId<Row>(read, input.id);
}

export function throughImportedBody(input: Input) {
  return withInput(write, input);
}

export function throughImportedDestructuring(input: Input) {
  return afterDestructuredExecutor(read, input);
}
