import fs from "node:fs";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
const { missing: replacement } = fs.readdir;
fs.readdir = replacement;
