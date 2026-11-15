const relationTables = Object.freeze({
  vote: "users",
  tag: "posts",
} as const);

export function getRelationTable(name: string) {
  return relationTables[name as keyof typeof relationTables];
}
