const relationTables = Object.freeze({
  vote: "users",
  tag: "posts",
} as const);

export function getRelationTable(name: string) {
  if (!Object.hasOwn(relationTables, name)) throw new Error(`unknown ${name}`);
  return relationTables[name as keyof typeof relationTables];
}

const relationConfigs = Object.freeze({
  vote: { table: "users", column: "user_id" },
  tag: { table: "posts", column: "post_id" },
} as const);

export function getConfig(name: string) {
  if (!Object.hasOwn(relationConfigs, name)) throw new Error(`unknown ${name}`);
  return relationConfigs[name as keyof typeof relationConfigs];
}

const loose = { vote: "users", tag: "posts" };

export function getLoose(name: string) {
  if (!Object.hasOwn(loose, name)) throw new Error(`unknown ${name}`);
  return loose[name as keyof typeof loose];
}
