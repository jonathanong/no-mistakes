const fs = require("node:fs"); // unrelated load: empty module selection does not select loaders
const db = require("@example/db"); // unrelated load without a configured module
fs.readFileSync("unrelated");
db.query("SELECT * FROM public.users"); // unsupported: explicitly configured broad member query
