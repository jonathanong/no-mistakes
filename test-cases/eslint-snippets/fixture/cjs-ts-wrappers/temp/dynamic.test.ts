const spec = "node:fs";
const loaded = require(spec);
loaded.writeFileSync("/tmp/a.txt", "a");
loaded.lstatSync("/tmp/a.txt");
