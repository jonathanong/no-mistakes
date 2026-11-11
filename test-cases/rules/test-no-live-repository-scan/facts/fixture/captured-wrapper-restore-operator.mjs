import fs from "node:fs";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
fs.readdir += original;
