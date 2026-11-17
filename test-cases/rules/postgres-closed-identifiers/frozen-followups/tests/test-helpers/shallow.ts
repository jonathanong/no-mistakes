const rec = Object.freeze({ vote: { table: "users", column: "id" } });

export function getConfig(key: string) {
  if (!Object.hasOwn(rec, key)) throw new Error("no");
  return rec[key as keyof typeof rec];
}
