// CommonJS shapes the call graph does not follow: each require is reported on its
// own line as an unresolved root, never silently skipped.
const { readdirSync: ls } = require("node:fs");
const read = require("node:fs").readdirSync;
const g = require("glob");

ls(".github"); // renamed destructured binding
read("docs"); // member of a required module
g.sync("**/*.yml"); // glob package namespace
