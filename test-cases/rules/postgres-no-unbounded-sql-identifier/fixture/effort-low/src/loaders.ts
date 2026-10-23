const { query } = require("@example/db"); // unsupported CommonJS load
query("SELECT * FROM public.users");
async function dynamic() {
  const db = await import("@example/db"); // unsupported dynamic module load
  return db.query("SELECT * FROM public.users");
}

const sub = require("@example/db/subpath"); // unsupported configured subpath load
sub.query("SELECT * FROM public.users");
require("node:fs"); // unrelated module
