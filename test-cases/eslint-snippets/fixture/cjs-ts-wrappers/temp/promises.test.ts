const spec = "node:fs";
const loaded = require(spec);
loaded.promises.writeFile("/tmp/a.txt", "a");
const { promises } = require(spec);
promises.writeFile("/tmp/b.txt", "b");
const { writeFile } = require(spec);
writeFile("/tmp/c.txt", "c");
