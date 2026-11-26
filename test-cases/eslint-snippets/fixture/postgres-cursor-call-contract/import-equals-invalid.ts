import cjs = require("@db/cursors");
export import reexported = require("@db/cursors");

export function invalidForms() {
  cjs.runCursor("SELECT 1");
  const fn = cjs.runCursor;
  void fn;
}

export { cjs };
