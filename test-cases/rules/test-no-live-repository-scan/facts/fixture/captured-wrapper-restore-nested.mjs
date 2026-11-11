import fs from "node:fs";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
const replacement = fs.promises.readdir;
fs.readdir = replacement;
