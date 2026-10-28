// Unsupported patterns must keep a load advisory instead of inventing targets.
const fallback = (_root: string) => [];
const key = process.env.METHOD;
const { readdirSync = fallback } = require("node:fs");
const { [key!]: computed } = require("node:fs");
const { 0: numeric } = require("node:fs"); // Unknown export keeps its load fallback.
const [first] = require("node:fs");
require("node:fs");
