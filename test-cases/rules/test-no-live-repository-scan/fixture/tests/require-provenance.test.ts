// Static CommonJS namespace and destructured bindings have independent roots.
const fs = require("node:fs");
fs.readdirSync("src");
fs.readdirSync("fixtures");
const { readdirSync } = require("node:fs");
readdirSync("packages");
readdirSync("fixtures");
const { readdirSync: ls } = require("node:fs");
ls("docs");
ls("fixtures");
