import fs from "node:fs";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
function reveal() { return fs.readdir; }
