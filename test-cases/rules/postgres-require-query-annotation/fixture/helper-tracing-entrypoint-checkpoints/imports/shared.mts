import { read } from "@app/db";
export function invoke(statement) { return read(statement); }
