import { COLS } from "./columns";

export function extend(extra: unknown) {
  COLS.append(extra);
}
