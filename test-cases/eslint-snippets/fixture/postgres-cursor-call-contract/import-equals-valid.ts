import cjs = require("@db/cursors");
import type typeOnly = require("@db/cursors");
import other = require("@other/cursors");

export type CursorFn = typeof cjs.runCursor;
export type TypeOnlyCursor = typeof typeOnly.runCursor;
export type { cjs };

export function validForms(dynamic: string) {
  cjs.runCursor("/* rows */ SELECT 1");
  cjs["runCursor"]("/* rows */ SELECT 1");
  cjs.other(dynamic);
  other.runCursor(dynamic);
}
