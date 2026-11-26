import { COLS } from "@example/columns";

export function extend(extra: unknown) {
  COLS.append(extra); // no-mistakes-disable-line postgres-sql-fragment-escape: fixture exception
}

export function extendFollowing(extra: unknown) {
  // no-mistakes-disable-next-line postgres-sql-fragment-escape: fixture exception
  COLS.append(extra);
}

export function extendControl(extra: unknown) {
  COLS.append(extra);
}
