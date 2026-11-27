const cur = require("@example/cursors");
const { runCursor: rc } = require("@example/cursors");
declare const dyn: string;

cur.runCursor(dyn);
rc(dyn);
require("@example/cursors").runCursor;
require("@example/cursors").runCursor(dyn);
