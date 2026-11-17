const relationTables = Object.freeze({
  vote: "users",
  tag: "posts",
} as const);

export function getRelationTable(name: string) {
  if (!Object.hasOwn(relationTables, name)) throw new Error(`unknown ${name}`);
  return relationTables[name as keyof typeof relationTables];
}

const relationConfigs = Object.freeze({
  vote: Object.freeze({ table: "users", column: "user_id" }),
  tag: Object.freeze({ table: "posts", column: "post_id" }),
} as const);

export function getConfig(name: string) {
  if (!Object.hasOwn(relationConfigs, name)) throw new Error(`unknown ${name}`);
  return relationConfigs[name as keyof typeof relationConfigs];
}
