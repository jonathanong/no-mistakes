import sql from "@example/sql";

declare const target: string;

export async function load() {
  const unrelated = sql`value`;
  await import(target);
  require(target);
  return typeof unrelated;
}
