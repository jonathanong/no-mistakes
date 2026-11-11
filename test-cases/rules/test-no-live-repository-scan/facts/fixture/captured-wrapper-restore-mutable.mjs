import fs from "node:fs";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
let replacement = original;
fs.readdir = replacement;
