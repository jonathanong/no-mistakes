// no-mistakes-disable-file postgres-sql-fragment-escape: fixture exception
import { COLS } from "@example/columns";

export function extend(extra: unknown) {
  COLS.append(extra);
}
