import pg from "pg";

const { Pool } = pg;
export const pool = new Pool({ host: "example.invalid" });
pool.on("error", console.error);

export function shadowModule(pg) {
  const { Pool } = pg;
  return new Pool({ host: "example.invalid" });
}

export function shadowConstructor(Pool) {
  return new Pool({ host: "example.invalid" });
}

export function localObject() {
  const pg = { Pool: class Pool {} };
  const { Pool } = pg;
  return new Pool({ host: "example.invalid" });
}
