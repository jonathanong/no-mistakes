import * as ns from "@example/cache";
import dbDefault from "@example/cache";
import eq = require("@example/cache");

const dbCjs = require("@example/cache");
const { db } = require("@example/cache");

export async function reads() {
  const a = await ns.db.get("a");
  const b = await ns.db.get("b");
  const c = await dbDefault.get("c");
  const d = await dbDefault.get("d");
  const e = await eq.db.get("e");
  const f = await eq.db.get("f");
  const g = await dbCjs.get("g");
  const h = await dbCjs.get("h");
  const i = await db.get("i");
  const j = await db.get("j");
  return [a, b, c, d, e, f, g, h, i, j];
}
