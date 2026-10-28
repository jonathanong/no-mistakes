// A resolved ESM scan must not hide a different CommonJS alias of the same module.
import { readdirSync } from "node:fs";
const { readdirSync: hidden } = require("node:fs");
readdirSync("docs");
hidden("src");

// Each plain CommonJS scan has its own root even when another scan is bounded.
const fs = require("node:fs");
fs.readdirSync("fixtures");
fs.readdirSync("packages");

// A computed module could be an enumerator; it must remain visible as unresolved.
const moduleName = "node:fs";
const dynamicFs = require(moduleName);
dynamicFs.readdirSync("scripts");
