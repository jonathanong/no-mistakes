const spec = "node:fs";
const method = "writeFileSync";
const loaded = require(spec);
loaded[method]("/tmp/result", "");
const { [method]: write } = require(spec);
write("/tmp/result", "");
