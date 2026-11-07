import { read } from "db";
import { getRelationTableNameOrThrow } from "./barrel.ts";

const PARENT_TABLE = getRelationTableNameOrThrow({
  subject: "topic", predicate: "parent", object: "topic",
});

export const listParents = (id: string) => read(
  `/* listParents */ SELECT object_id FROM ${PARENT_TABLE} WHERE subject_id = $1 AND deleted_at IS NULL`, [id]);

export function localParents(id: string) {
  const table = getRelationTableNameOrThrow({ subject: "topic", predicate: "parent", object: "topic" });
  return read(`/* localParents */ SELECT object_id FROM ${table} WHERE subject_id = $1`, [id]);
}

export function inlineParents(id: string) {
  return read(`SELECT object_id FROM ${getRelationTableNameOrThrow({ subject: "topic", predicate: "parent", object: "topic" })} WHERE subject_id = $1`, [id]);
}

export async function awaitedParents() {
  return read(`SELECT object_id FROM ${await getRelationTableNameOrThrow({ subject: "topic", predicate: "parent", object: "topic" })}`);
}

export function conditionalParents(flag: boolean) {
  return read(`SELECT object_id FROM ${flag ? getRelationTableNameOrThrow({ subject: "topic", predicate: "parent", object: "topic" }) : "items"}`);
}
