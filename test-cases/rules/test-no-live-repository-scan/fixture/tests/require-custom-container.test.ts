// The configured scanner is below each destructured container.
const { ns } = require("walker");
let scan = ns.scan;
scan("src");
const { readFileSync: maybe } = require("node:fs");
let child = maybe.child;
child("src");
