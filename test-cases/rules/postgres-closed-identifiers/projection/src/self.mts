import { query } from "@example/db";
import { getSelf as importedSelf } from "./self.mts";

const relationConfigs = Object.freeze({
  vote: { table: "users", column: "user_id" },
  tag: { table: "posts", column: "post_id" },
} as const);

export function getSelf(name: string) {
  if (!Object.hasOwn(relationConfigs, name)) throw new Error(`unknown ${name}`);
  return relationConfigs[name as keyof typeof relationConfigs];
}

export function run(kind: string) {
  const config = importedSelf(kind);
  query(`/* self */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}
