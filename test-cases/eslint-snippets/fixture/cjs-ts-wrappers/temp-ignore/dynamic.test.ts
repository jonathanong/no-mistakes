const spec = "node:fs";
const loaded = require(spec);
loaded.writeFileSync("/tmp/a.txt", "a");
