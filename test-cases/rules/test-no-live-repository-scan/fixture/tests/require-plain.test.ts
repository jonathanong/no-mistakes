// Requires that need no report: a resolved enumerator use and unrelated modules.
const path = require("node:path");
const fs = require("node:fs");
const walker = require("walker");

fs.readdirSync(path.join(__dirname, "fixtures"));
walker("x");
