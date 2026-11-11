import fs from "node:fs";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
fs.readdir = replacement;
const replacement = fs.readdir;
