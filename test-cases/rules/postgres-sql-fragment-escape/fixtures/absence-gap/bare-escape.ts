import { COLS } from "@example/columns";

export function extend(extra: unknown) {
  COLS.append(extra);
}
