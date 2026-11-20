import * as external from "pkg/sql";
import { localColumns } from "./local/sql";

export function extend(extra: unknown) {
  Object.assign(external, { extra });
  localColumns.append(extra);
}
