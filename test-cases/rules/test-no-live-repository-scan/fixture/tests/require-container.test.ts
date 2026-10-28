// A destructured namespace can still expose a scanner through a later alias.
let { promises: fsPromises } = require("node:fs");
const scan = fsPromises.readdir;
scan("src");
// Unknown export names are not proof that a module cannot enumerate.
const { 0: unknown } = require("node:fs");
void unknown;

const { promises: direct } = require("node:fs");
direct.readdir("src");
direct.readdir("fixtures");
const { escapePath: escape } = require("fast-glob");
escape("src");
