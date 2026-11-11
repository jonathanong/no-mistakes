import fs from "fs-extra";
const original = fs.readdir;
fs.readdir = (...args) => original(...args);
