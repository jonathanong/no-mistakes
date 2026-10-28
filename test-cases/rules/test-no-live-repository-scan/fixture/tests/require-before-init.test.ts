// A direct module-scope call before the const initializer stays unresolved.
fs.readdirSync("src");
const fs = require("node:fs");
