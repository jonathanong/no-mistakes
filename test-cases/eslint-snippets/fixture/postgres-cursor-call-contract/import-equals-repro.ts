import * as esm from "@db/cursors";
import cjs = require("@db/cursors");

export function inspect(dynamic: string, key: "runCursor") {
  esm.runCursor(dynamic);
  cjs.runCursor(dynamic);
  esm[key](dynamic);
  cjs[key](dynamic);
  const { runCursor: esmCursor } = esm;
  const { runCursor: cjsCursor } = cjs;
  esmCursor(dynamic);
  cjsCursor(dynamic);
}
