import fs from "node:fs/promises";
const original = fs.readdir;
fs.readdir = (...args) => {
  if (args[0]) throw new Error("injected path failure");
  if (args.length) throw new Error("injected arity failure");
  return original(...args);
};
